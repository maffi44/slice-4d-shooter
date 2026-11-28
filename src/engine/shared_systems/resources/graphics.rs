// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use super::*;
use crate::engine::shared_systems::render::pipelines;
use crate::engine::world::world_descriptor::{parse_json_level, LevelData};
use fyrox_resource::loader::ResourceLoadersContainer;
pub use pipelines::BuiltinPipelineType;
use serde::Deserialize;

macro_rules! runtime_asset {
    ($name:ident, $data:ty, $uuid:literal) => {
        #[derive(Default, Visit, Reflect)]
        #[reflect(non_cloneable)]
        pub struct $name(
            #[visit(skip)]
            #[reflect(hidden)]
            Option<$data>,
        );
        impl $name {
            pub fn value(&self) -> &$data {
                self.0.as_ref().expect("ready runtime asset")
            }
        }
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(stringify!($name))
            }
        }
        impl TypeUuidProvider for $name {
            fn type_uuid() -> Uuid {
                uuid!($uuid)
            }
        }
        impl ResourceData for $name {
            fn type_uuid(&self) -> Uuid {
                <Self as TypeUuidProvider>::type_uuid()
            }
            fn can_be_saved(&self) -> bool {
                false
            }
            fn save(&mut self, _: &Path) -> Result<(), Box<dyn Error>> {
                Err("Runtime asset".into())
            }
            fn try_clone_box(&self) -> Option<Box<dyn ResourceData>> {
                None
            }
        }
    };
}
runtime_asset!(
    LevelResource,
    Arc<LevelData>,
    "87036f50-e662-4490-95fe-a391a90123b8"
);
runtime_asset!(
    RenderPipelineResource,
    wgpu::RenderPipeline,
    "2c770ff8-5405-4cad-be59-d0db1cd13388"
);
runtime_asset!(
    CubemapTexture,
    wgpu::TextureView,
    "16acd5cc-20a5-4fcb-a245-0d6062adf9d8"
);

impl Asset for LevelResource {
    fn manager_path(key: &str) -> Result<Cow<'_, Path>, String> {
        Ok(Cow::Owned(
            WorldResource::manager_path(key)?.with_extension("level"),
        ))
    }
}
impl Asset for RenderPipelineResource {
    const REQUIRES_GPU: bool = true;
    fn manager_path(key: &str) -> Result<Cow<'_, Path>, String> {
        let key = key
            .strip_suffix(".pipeline")
            .ok_or("Expected a .pipeline resource")?;
        if let Some(specialization) = key.strip_suffix(".bsp") {
            let (map, shader) = specialization
                .rsplit_once('.')
                .ok_or("Missing level pipeline variant")?;
            let shader = BuiltinPipelineType::parse(shader)
                .filter(|s| *s != BuiltinPipelineType::Scanner)
                .ok_or("Unknown level pipeline")?;
            let path = WorldResource::manager_path(map)?.with_extension("json");
            let map = path.to_str().ok_or("Invalid level path")?;
            Ok(Cow::Owned(PathBuf::from(format!(
                "{map}.{}.bsp.pipeline",
                shader.name()
            ))))
        } else if key == BuiltinPipelineType::Scanner.name() {
            Ok(Cow::Borrowed(Path::new("scanner.pipeline")))
        } else {
            Err(format!("Pipeline {key} requires level geometry"))
        }
    }
}
impl Asset for CubemapTexture {
    const REQUIRES_GPU: bool = true;
    fn manager_path(key: &str) -> Result<Cow<'_, Path>, String> {
        let p = logical_path(key, "embedded");
        if p.extension().is_some_and(|e| e == "cubemap") {
            Ok(Cow::Borrowed(p))
        } else {
            Err(format!("Expected a .cubemap descriptor: {key}"))
        }
    }
}
impl RenderPipelineResource {
    pub fn request(
        loader: &mut ResourceLoader,
        shader: BuiltinPipelineType,
        level: Option<&str>,
    ) -> Resource<Self> {
        let key = match level {
            Some(map) => format!("{map}.{}.bsp.pipeline", shader.name()),
            None => format!("{}.pipeline", shader.name()),
        };
        loader.load::<Self>(&key, Retention::WhileUsed)
    }
}

struct LevelLoader(Handle);
impl FyroxResourceLoader for LevelLoader {
    fn extensions(&self) -> &[&str] {
        &["level"]
    }
    fn data_type_uuid(&self) -> Uuid {
        <LevelResource as TypeUuidProvider>::type_uuid()
    }
    fn load(&self, path: PathBuf, _: Arc<dyn ResourceIo>) -> BoxedLoaderFuture {
        load_on_runtime(&self.0, async move {
            let file = path.with_extension("json");
            let paths = [
                ".",
                "./assets/maps",
                "./src/assets/maps",
                "../../../src/assets/maps",
            ]
            .map(|dir| Path::new(dir).join(&file));
            let source = source::read(
                &Path::new("maps").join(&file),
                paths,
                source::Lookup::DiskFirst,
            )
            .await?;
            let origin = source.origin.clone();
            tokio::task::spawn_blocking(move || {
                let json = serde_json::from_slice(&source.bytes)
                    .map_err(|e| format!("{}: Wrong JSON map format: {e}", source.origin))?;
                Ok(LoaderPayload::new(LevelResource(Some(Arc::new(
                    parse_json_level(json, source.origin),
                )))))
            })
            .await
            .map_err(|e| format!("{origin}: {e}"))?
        })
    }
}

type DeviceReceiver = watch::Receiver<Option<GpuContext>>;
async fn device(mut receiver: DeviceReceiver) -> Result<GpuContext, String> {
    let gpu = receiver
        .wait_for(Option::is_some)
        .await
        .map_err(|_| "GPU initialization ended before device publication")?;
    Ok(gpu.as_ref().unwrap().clone())
}
struct PipelineLoader {
    runtime: Handle,
    gpu: DeviceReceiver,
    resources: Weak<ResourceSystem>,
}
impl FyroxResourceLoader for PipelineLoader {
    fn extensions(&self) -> &[&str] {
        &["pipeline"]
    }
    fn data_type_uuid(&self) -> Uuid {
        <RenderPipelineResource as TypeUuidProvider>::type_uuid()
    }
    fn load(&self, path: PathBuf, _: Arc<dyn ResourceIo>) -> BoxedLoaderFuture {
        let gpu = self.gpu.clone();
        let system = self.resources.clone();
        load_on_runtime(&self.runtime, async move {
            let key = path
                .to_str()
                .and_then(|s| s.strip_suffix(".pipeline"))
                .ok_or("Invalid pipeline key")?;
            let (level, shader) = if let Some(prefix) = key.strip_suffix(".bsp") {
                let (map, shader) = prefix.rsplit_once('.').ok_or("Missing pipeline variant")?;
                let system = system.upgrade().ok_or("Engine has shut down")?;
                let data = system
                    .loader()
                    .load::<LevelResource>(map, Retention::WhileUsed)
                    .await
                    .map_err(|e| format!("{e:?}"))?;
                let level = data.data_ref().value().clone();
                (Some(level), shader)
            } else {
                (None, key)
            };
            let shader = BuiltinPipelineType::parse(shader)
                .ok_or_else(|| format!("Unknown render pipeline {key}"))?;
            let gpu = device(gpu).await?;
            let (pipeline, validation) = tokio::task::spawn_blocking(move || {
                let scope = gpu.device.push_error_scope(wgpu::ErrorFilter::Validation);
                let pipeline = pipelines::compile(&gpu, shader, level.as_deref());
                (pipeline, scope.pop())
            })
            .await
            .map_err(|e| e.to_string())?;
            if let Some(e) = validation.await {
                return Err(format!("{key}: {e}"));
            }
            Ok(LoaderPayload::new(RenderPipelineResource(Some(pipeline))))
        })
    }
}

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkyFilter {
    Nearest,
    #[default]
    Linear,
}
impl SkyFilter {
    pub fn sampler(self, device: &wgpu::Device) -> wgpu::Sampler {
        let filter = match self {
            Self::Linear => wgpu::FilterMode::Linear,
            Self::Nearest => wgpu::FilterMode::Nearest,
        };
        device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("skybox sampler"),
            mag_filter: filter,
            min_filter: filter,
            ..Default::default()
        })
    }
}
pub struct Skybox {
    pub path: String,
    pub filter: SkyFilter,
}
impl Skybox {
    pub fn parse(env: &serde_json::Value) -> Result<Self, String> {
        let path = match env
            .get("sky_box")
            .map(|v| v.as_str().ok_or("sky_box must be a string"))
            .transpose()?
        {
            None | Some("star_sky") => {
                crate::embedded_asset!("sky_boxes/star_sky/star_sky.cubemap")
            }
            Some(path) => path,
        };
        let filter = env
            .get("sky_box_filter")
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()
            .map_err(|e| format!("sky_box_filter: {e}"))?
            .unwrap_or_default();
        Ok(Self {
            path: path.to_owned(),
            filter,
        })
    }
}

struct CubeLoader {
    runtime: Handle,
    gpu: DeviceReceiver,
}
#[derive(Deserialize)]
struct Faces {
    positive_x: String,
    negative_x: String,
    positive_y: String,
    negative_y: String,
    positive_z: String,
    negative_z: String,
}
impl FyroxResourceLoader for CubeLoader {
    fn extensions(&self) -> &[&str] {
        &["cubemap"]
    }
    fn data_type_uuid(&self) -> Uuid {
        <CubemapTexture as TypeUuidProvider>::type_uuid()
    }
    fn load(&self, path: PathBuf, _: Arc<dyn ResourceIo>) -> BoxedLoaderFuture {
        let gpu = self.gpu.clone();
        load_on_runtime(&self.runtime, async move {
            let src = source::read(&path, [&path], source::Lookup::CatalogFirst).await?;
            let faces: Faces =
                serde_json::from_slice(&src.bytes).map_err(|e| format!("{}: {e}", src.origin))?;
            let mut sources = Vec::with_capacity(6);
            for file in [
                faces.positive_x,
                faces.negative_x,
                faces.positive_y,
                faces.negative_y,
                faces.positive_z,
                faces.negative_z,
            ] {
                let face = path.parent().unwrap_or(Path::new("")).join(file);
                sources.push(source::read(&face, [&face], source::Lookup::CatalogFirst).await?);
            }
            let gpu = device(gpu).await?;
            tokio::task::spawn_blocking(move || {
                let mut images = Vec::with_capacity(6);
                let mut width = 0;
                for src in sources {
                    let image = image::load_from_memory(&src.bytes)
                        .map_err(|e| format!("{}: {e}", src.origin))?
                        .into_rgba8();
                    if image.width() != image.height()
                        || (width != 0 && width != image.width())
                        || image.width() > gpu.device.limits().max_texture_dimension_2d
                    {
                        return Err(format!(
                            "{}: incompatible cubemap face dimensions {:?}",
                            src.origin,
                            image.dimensions()
                        ));
                    }
                    width = image.width();
                    images.push(image);
                }
                let size = wgpu::Extent3d {
                    width,
                    height: width,
                    depth_or_array_layers: 6,
                };
                let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
                    label: path.to_str(),
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                for (index, image) in images.iter().enumerate() {
                    gpu.queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            origin: wgpu::Origin3d {
                                x: 0,
                                y: 0,
                                z: index as u32,
                            },
                            ..texture.as_image_copy()
                        },
                        image,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(width * 4),
                            rows_per_image: Some(width),
                        },
                        wgpu::Extent3d {
                            depth_or_array_layers: 1,
                            ..size
                        },
                    );
                }
                Ok(LoaderPayload::new(CubemapTexture(Some(
                    texture.create_view(&wgpu::TextureViewDescriptor {
                        dimension: Some(wgpu::TextureViewDimension::Cube),
                        ..Default::default()
                    }),
                ))))
            })
            .await
            .map_err(|e| e.to_string())?
        })
    }
}
pub(super) fn register(
    loaders: &mut ResourceLoadersContainer,
    runtime: Handle,
    gpu: Option<DeviceReceiver>,
    resources: Weak<ResourceSystem>,
) {
    loaders.set(LevelLoader(runtime.clone()));
    if let Some(gpu) = gpu {
        loaders.set(PipelineLoader {
            runtime: runtime.clone(),
            gpu: gpu.clone(),
            resources,
        });
        loaders.set(CubeLoader { runtime, gpu });
    }
}
