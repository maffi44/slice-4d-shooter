// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use super::{
    texture_view, BuiltinPipeline, Draw, RenderPipelineResource, ResourceLoader, TextureDraw,
    Uniform, WorldRenderSystem, WorldTextureId,
};
use glam::UVec2;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Weak,
};
use tokio::{sync::watch, task::JoinHandle};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LocalImageKey {
    owner: WorldTextureId,
    id: u64,
    pub(crate) size: UVec2,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ImageSource {
    Local(LocalImageKey),
    World(WorldTextureId),
}
impl From<LocalImageKey> for ImageSource {
    fn from(key: LocalImageKey) -> Self {
        Self::Local(key)
    }
}
impl From<WorldTextureId> for ImageSource {
    fn from(id: WorldTextureId) -> Self {
        Self::World(id)
    }
}
impl ImageSource {
    pub(crate) fn can_sample_from(self, consumer: WorldTextureId) -> bool {
        match self {
            Self::Local(key) => key.owner == consumer,
            Self::World(source) => source.renders_before(consumer),
        }
    }
}

#[repr(u32)]
pub enum ScannerPlane {
    Zx,
    Zw,
}
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ScannerParams {
    padding: [u32; 3],
    orientation: u32,
}

pub enum TextureEffect {
    Scanner(ScannerPlane),
    XwzMinimap,
}

enum TextureInputs {
    Scanner { plane: ScannerPlane, world: Uniform<crate::engine::shared_systems::render::render_data::dynamic_render_data::OtherDynamicData> },
    Scene(Vec<wgpu::BindGroup>),
}

type PreparedTexture = Result<Arc<TextureDraw>, String>;
pub(super) struct EffectInstance {
    pub(super) key: LocalImageKey,
    pub(super) state: watch::Receiver<Option<PreparedTexture>>,
    task: JoinHandle<()>,
    pub(super) owned: AtomicBool,
}
impl Drop for EffectInstance {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// The actor owns production. Widgets only keep a non-owning image key.
pub struct DynamicTexture {
    key: LocalImageKey,
    instance: Weak<EffectInstance>,
    dirty: Arc<AtomicBool>,
}
impl DynamicTexture {
    pub fn image(&self) -> LocalImageKey {
        self.key
    }
    #[allow(dead_code)] // Actors may poll while the world keeps simulating.
    pub fn try_ready(&self) -> Option<Result<(), String>> {
        let Some(instance) = self.instance.upgrade() else {
            return Some(Err("World was dropped".into()));
        };
        let state = instance.state.borrow();
        state
            .as_ref()
            .map(|result| result.as_ref().map(|_| ()).map_err(Clone::clone))
    }
    #[allow(dead_code)] // Loading callers can wait independently of simulation and rendering.
    pub async fn ready(&self) -> Result<(), String> {
        let mut state = self
            .instance
            .upgrade()
            .ok_or("World was dropped")?
            .state
            .clone();
        let ready = state
            .wait_for(Option::is_some)
            .await
            .map_err(|_| "Texture preparation ended")?;
        ready
            .as_ref()
            .unwrap()
            .as_ref()
            .map(|_| ())
            .map_err(Clone::clone)
    }
}
impl Drop for DynamicTexture {
    fn drop(&mut self) {
        if let Some(instance) = self.instance.upgrade() {
            instance.owned.store(false, Ordering::Release);
            instance.task.abort();
            self.dirty.store(true, Ordering::Release);
        }
    }
}

impl WorldRenderSystem {
    pub fn create_dynamic_texture(
        &mut self,
        loader: &mut ResourceLoader,
        effect: TextureEffect,
        size: UVec2,
    ) -> DynamicTexture {
        let id = self.next_effect;
        self.next_effect += 1;
        let key = LocalImageKey {
            owner: self.texture_id,
            id,
            size,
        };
        let (shader, level, inputs) = match effect {
            TextureEffect::Scanner(plane) => (
                BuiltinPipeline::Scanner,
                None,
                TextureInputs::Scanner {
                    plane,
                    world: self.buffers.other_dynamic_data_buffer.clone(),
                },
            ),
            TextureEffect::XwzMinimap => (
                BuiltinPipeline::XwzMinimap,
                Some(self.level_key.as_str()),
                TextureInputs::Scene(self.scene.groups.clone()),
            ),
        };
        let resource = RenderPipelineResource::request(loader, shader, level);
        let gpu = self.gpu.clone();
        let (sender, state) = watch::channel(None);
        let dirty = self.dirty.clone();
        let task = self.runtime.spawn(async move {
            let result = async {
                let pipeline = resource.await.map_err(|e| format!("{e:?}"))?;
                let (draw, validation) = tokio::task::spawn_blocking(move || {
                    let limit = gpu.device.limits().max_texture_dimension_2d;
                    if size.min_element() == 0 || size.max_element() > limit {
                        return Err(format!("Invalid texture size {size}"));
                    }
                    let scope = gpu.device.push_error_scope(wgpu::ErrorFilter::Validation);
                    let groups = match inputs {
                        TextureInputs::Scanner { plane, world } => {
                            let params = Uniform::new(
                                &gpu.device,
                                "scanner plane",
                                &ScannerParams {
                                    padding: [0; 3],
                                    orientation: plane as u32,
                                },
                            );
                            vec![gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                                label: Some("scanner"),
                                layout: &gpu.scanner_layout,
                                entries: &[
                                    wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: world.binding(),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 1,
                                        resource: params.binding(),
                                    },
                                ],
                            })]
                        }
                        TextureInputs::Scene(groups) => groups,
                    };
                    let draw = Draw {
                        pipeline: pipeline.data_ref().value().clone(),
                        groups,
                    };
                    Ok((
                        Arc::new(TextureDraw {
                            draw,
                            _pipeline: pipeline,
                            view: texture_view(&gpu, size, "dynamic image"),
                        }),
                        scope.pop(),
                    ))
                })
                .await
                .map_err(|e| e.to_string())??;
                if let Some(error) = validation.await {
                    return Err(error.to_string());
                }
                Ok(draw)
            }
            .await;
            if let Err(err) = &result {
                log::error!("Dynamic texture preparation failed: {err}");
            }
            sender.send_replace(Some(result));
            dirty.store(true, Ordering::Release);
        });
        let instance = Arc::new(EffectInstance {
            key,
            state,
            task,
            owned: AtomicBool::new(true),
        });
        let handle = DynamicTexture {
            key,
            instance: Arc::downgrade(&instance),
            dirty: self.dirty.clone(),
        };
        self.effects.push(instance);
        self.dirty.store(true, Ordering::Release);
        handle
    }
}
