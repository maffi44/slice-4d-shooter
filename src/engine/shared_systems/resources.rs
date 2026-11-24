// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use fyrox_core::{
    parking_lot::Mutex,
    reflect::prelude::*,
    uuid::{uuid, Uuid},
    visitor::prelude::*,
    TypeUuidProvider,
};
use fyrox_resource::{
    io::ResourceIo,
    loader::{BoxedLoaderFuture, LoaderPayload, ResourceLoader as FyroxResourceLoader},
    manager::ResourceManager,
    state::LoadError,
    untyped::ResourceKind,
    untyped::UntypedResource,
    Resource, ResourceData, TypedResourceData,
};
use std::{
    borrow::Cow,
    collections::HashSet,
    error::Error,
    future::Future,
    path::{Path, PathBuf},
    sync::{Arc, Weak},
};

use super::render::GpuContext;
use crate::engine::world::world_descriptor::{WorldDescriptor, WorldPreparation};

use fyrox_sound::buffer::{DataSource, SoundBuffer};
use tokio::{runtime::Handle, sync::watch};

mod source;

/// Build-time marker and logical key for an asset under src/assets.
#[macro_export]
macro_rules! embedded_asset {
    ($path:literal) => {
        concat!("embedded/", $path)
    };
}

/// Maps a caller's typed key to the path Fyrox uses for cache identity and loader selection.
pub trait Asset: TypedResourceData {
    const REQUIRES_GPU: bool = false;
    fn manager_path(key: &str) -> Result<Cow<'_, Path>, String>;
}

fn logical_path<'a>(key: &'a str, embedded_root: &str) -> &'a Path {
    let path = Path::new(key);
    let path = path.strip_prefix(".").unwrap_or(path);
    path.strip_prefix(embedded_root).unwrap_or(path)
}

impl Asset for SoundBuffer {
    fn manager_path(key: &str) -> Result<Cow<'_, Path>, String> {
        let path = logical_path(key, "embedded");
        if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("wav"))
        {
            Ok(Cow::Borrowed(path))
        } else {
            Err(format!("Expected a WAV sound: {key}"))
        }
    }
}

/// The Fyrox pool polls this join handle. File I/O and blocking jobs belong to the engine runtime.
fn load_on_runtime(
    runtime: &Handle,
    work: impl Future<Output = Result<LoaderPayload, String>> + Send + 'static,
) -> BoxedLoaderFuture {
    let task = runtime.spawn(work);
    Box::pin(async move {
        task.await
            .map_err(|error| LoadError::new(error.to_string()))?
            .map_err(LoadError::new)
    })
}

struct SoundLoader(Handle);

impl FyroxResourceLoader for SoundLoader {
    fn extensions(&self) -> &[&str] {
        &["wav"]
    }
    fn data_type_uuid(&self) -> Uuid {
        <SoundBuffer as TypeUuidProvider>::type_uuid()
    }
    fn load(&self, path: PathBuf, _: Arc<dyn ResourceIo>) -> BoxedLoaderFuture {
        load_on_runtime(&self.0, async move {
            let source = source::read(&path, [&path], source::Lookup::CatalogFirst).await?;
            tokio::task::spawn_blocking(move || {
                SoundBuffer::raw_generic(DataSource::from_memory(source.bytes.into_owned()))
                    .map(LoaderPayload::new)
                    .map_err(|_| format!("Invalid sound {}", source.origin))
            })
            .await
            .map_err(|error| error.to_string())?
        })
    }
}

/// Actor-facing access to the shared cache. Only map preparation lends a dependency list.
pub struct ResourceLoader<'a> {
    system: Arc<ResourceSystem>,
    dependencies: Option<&'a mut Vec<UntypedResource>>,
}

impl ResourceLoader<'_> {
    /// Returns a shared pending, ready or failed handle immediately. Maps search disk before
    /// the catalog. Other assets search the catalog first. Retention also applies to cache hits.
    pub fn load<T: Asset>(&mut self, key: &str, retention: Retention) -> Resource<T> {
        let resource = match T::manager_path(key) {
            Ok(path) if T::REQUIRES_GPU && self.system.headless => Resource::new_load_error(
                ResourceKind::External(path.into_owned()),
                LoadError::new(format!("Resource {key} requires a GPU in headless mode")),
            ),
            Ok(path) => {
                let resource: Resource<T> = self.system.manager.request(path.as_ref());
                if matches!(retention, Retention::Session) {
                    self.system
                        .retained
                        .lock()
                        .insert(resource.clone().into_untyped());
                }
                resource
            }
            Err(error) => {
                Resource::new_load_error(ResourceKind::External(key.into()), LoadError::new(error))
            }
        };
        if let Some(pending) = &mut self.dependencies {
            pending.push(resource.clone().into_untyped());
        }
        resource
    }

    /// Headless callers skip optional media before performing any work or recording dependencies.
    pub fn load_media<T: Asset>(&mut self, key: &str, retention: Retention) -> Option<Resource<T>> {
        (!self.system.headless).then(|| self.load(key, retention))
    }
}

/// Inspect a typed handle once during a tick. Contention can wait for the next tick.
pub trait ResourceRead<T> {
    fn try_read<R>(&self, read: impl FnOnce(&T) -> R) -> std::task::Poll<Result<R, LoadError>>;
}

impl<T: TypedResourceData> ResourceRead<T> for Resource<T> {
    fn try_read<R>(&self, read: impl FnOnce(&T) -> R) -> std::task::Poll<Result<R, LoadError>> {
        use fyrox_resource::state::ResourceState;
        use std::task::Poll;
        let handle = self.clone().into_untyped();
        let Some(header) = handle.0.try_lock() else {
            return Poll::Pending;
        };
        match &header.state {
            ResourceState::Pending { .. } => Poll::Pending,
            ResourceState::LoadError { error } => Poll::Ready(Err(error.clone())),
            ResourceState::Ok(data) => {
                let typed = fyrox_core::Downcast::as_any(&**data)
                    .downcast_ref::<T>()
                    .expect("Resource data must match the typed handle");
                Poll::Ready(Ok(read(typed)))
            }
        }
    }
}

pub enum Retention {
    Session,
    WhileUsed,
}

pub(in crate::engine) struct ResourceSystem {
    manager: ResourceManager,
    retained: Mutex<HashSet<UntypedResource>>,
    headless: bool,
}

impl ResourceSystem {
    /// A graphical loader publishes the device once. None selects headless preparation.
    pub fn new(runtime: Handle, gpu: Option<watch::Receiver<Option<GpuContext>>>) -> Arc<Self> {
        Arc::new_cyclic(|resources| {
            let manager = ResourceManager::new(Default::default());
            let headless = gpu.is_none();
            manager.state().loaders.set(SoundLoader(runtime.clone()));
            manager.state().loaders.set(WorldLoader {
                resources: resources.clone(),
                gpu,
                runtime,
            });
            Self {
                manager,
                retained: Default::default(),
                headless,
            }
        })
    }

    pub fn loader(self: &Arc<Self>) -> ResourceLoader<'static> {
        ResourceLoader {
            system: Arc::clone(self),
            dependencies: None,
        }
    }

    pub fn loader_with_dependencies<'a>(
        self: &Arc<Self>,
        dependencies: &'a mut Vec<UntypedResource>,
    ) -> ResourceLoader<'a> {
        ResourceLoader {
            system: Arc::clone(self),
            dependencies: Some(dependencies),
        }
    }

    pub fn update(&self, delta: f32) {
        self.manager.state().update(delta);
    }
}

impl Asset for WorldResource {
    fn manager_path(key: &str) -> Result<Cow<'_, Path>, String> {
        let path = logical_path(key, "embedded/maps");
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("Expected a map name: {key}"))?;
        let name = name.strip_suffix(".json").unwrap_or(name);
        if name.is_empty() {
            return Err(format!("Expected a map name: {key}"));
        }
        Ok(Cow::Owned(path.with_file_name(format!("{name}.world"))))
    }
}

#[derive(Default, Visit, Reflect)]
pub struct WorldResource(
    #[visit(skip)]
    #[reflect(hidden)]
    pub Option<Arc<WorldDescriptor>>,
);

impl std::fmt::Debug for WorldResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("WorldResource")
            .field(&self.0.as_ref().map(|world| &world.name))
            .finish()
    }
}

impl TypeUuidProvider for WorldResource {
    fn type_uuid() -> Uuid {
        uuid!("284cc71c-5e62-4d58-bae6-d4f16ac814ee")
    }
}

impl ResourceData for WorldResource {
    fn type_uuid(&self) -> Uuid {
        <Self as TypeUuidProvider>::type_uuid()
    }
    fn can_be_saved(&self) -> bool {
        false
    }
    fn save(&mut self, _: &Path) -> Result<(), Box<dyn Error>> {
        Err("Runtime world descriptor".into())
    }
}

struct WorldLoader {
    resources: Weak<ResourceSystem>,
    gpu: Option<watch::Receiver<Option<GpuContext>>>,
    runtime: Handle,
}

impl FyroxResourceLoader for WorldLoader {
    fn extensions(&self) -> &[&str] {
        &["world"]
    }
    fn data_type_uuid(&self) -> Uuid {
        <WorldResource as TypeUuidProvider>::type_uuid()
    }
    fn load(&self, path: PathBuf, _: Arc<dyn ResourceIo>) -> BoxedLoaderFuture {
        let resources = self.resources.clone();
        let gpu = self.gpu.clone();
        load_on_runtime(&self.runtime, async move {
            let resources = resources.upgrade().ok_or("Engine has shut down")?;
            let file = path.with_extension("json");
            let paths = [
                ".",
                "./assets/maps",
                "./src/assets/maps",
                "../../../src/assets/maps",
            ]
            .map(|dir| Path::new(dir).join(&file));
            let key = Path::new("maps").join(&file);
            let source = source::read(&key, paths, source::Lookup::DiskFirst).await?;
            let prepared = WorldPreparation::from_bytes(source.bytes, resources)
                .await
                .map_err(|error| format!("{}: {error}", source.origin))?;
            let device = match gpu {
                Some(mut receiver) => receiver
                    .wait_for(Option::is_some)
                    .await
                    .map_err(|_| "GPU initialization ended before device publication")?
                    .clone(),
                None => None,
            };
            let descriptor = tokio::task::spawn_blocking(move || prepared.finish(device))
                .await
                .map_err(|error| format!("{}: {error}", source.origin))?;
            Ok(LoaderPayload::new(WorldResource(Some(Arc::new(
                descriptor,
            )))))
        })
    }
}

#[cfg(test)]
mod tests;
