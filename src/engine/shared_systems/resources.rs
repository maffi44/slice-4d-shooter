// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use fyrox_core::{
    TypeUuidProvider,
    parking_lot::Mutex,
    reflect::prelude::*,
    uuid::{Uuid, uuid},
    visitor::prelude::*,
};
use fyrox_resource::{
    Resource, ResourceData, TypedResourceData,
    io::ResourceIo,
    loader::{BoxedLoaderFuture, LoaderPayload, ResourceLoader},
    manager::ResourceManager,
    state::LoadError,
    untyped::ResourceKind,
    untyped::UntypedResource,
};
use std::{
    collections::HashSet,
    error::Error,
    path::{Path, PathBuf},
    sync::{Arc, Weak},
};

use super::render::GpuContext;
use crate::engine::world::world_descriptor::{WorldDescriptor, WorldPreparation};

use fyrox_sound::buffer::{DataSource, SoundBuffer, SoundBufferResource};

/// Build-time marker and logical key for an asset under src/assets.
#[macro_export]
macro_rules! embedded_asset {
    ($path:literal) => {
        concat!("embedded/", $path)
    };
}

include!(concat!(env!("OUT_DIR"), "/embedded_assets.rs"));

struct SoundLoader;

impl ResourceLoader for SoundLoader {
    fn extensions(&self) -> &[&str] {
        &["wav"]
    }
    fn data_type_uuid(&self) -> Uuid {
        <SoundBuffer as TypeUuidProvider>::type_uuid()
    }
    fn load(&self, path: PathBuf, io: Arc<dyn ResourceIo>) -> BoxedLoaderFuture {
        let bytes = path.to_str().and_then(embedded_bytes);
        Box::pin(async move {
            let source = match bytes {
                Some(bytes) => DataSource::from_memory(bytes.to_vec()),
                None => DataSource::from_file(&path, io.as_ref())
                    .await
                    .map_err(LoadError::new)?,
            };
            SoundBuffer::raw_generic(source)
                .map(LoaderPayload::new)
                .map_err(|_| LoadError::new(format!("Invalid sound {}", path.display())))
        })
    }
}

/// Initial world preparation collects exactly the handles requested by its constructors.
pub struct ResourceRequests<'a> {
    system: &'a ResourceSystem,
    dependencies: Option<Vec<UntypedResource>>,
}

impl ResourceRequests<'_> {
    pub fn sound(
        &mut self,
        key: &'static str,
        retention: Retention,
    ) -> Option<SoundBufferResource> {
        if self.system.headless {
            return None;
        }
        let path = Path::new(key);
        let buffer = self.system.request::<SoundBuffer>(path, retention);
        if let Some(pending) = &mut self.dependencies {
            pending.push(buffer.clone().into_untyped());
        }
        Some(buffer)
    }

    pub fn into_dependencies(self) -> Vec<UntypedResource> {
        self.dependencies.unwrap_or_default()
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

pub struct ResourceSystem {
    manager: ResourceManager,
    retained: Mutex<HashSet<UntypedResource>>,
    pub headless: bool,
}

impl ResourceSystem {
    pub fn new(headless: bool) -> Self {
        let manager = ResourceManager::new(Default::default());
        manager.state().loaders.set(SoundLoader);
        Self {
            manager,
            retained: Default::default(),
            headless,
        }
    }

    pub fn requests(&self) -> ResourceRequests<'_> {
        ResourceRequests {
            system: self,
            dependencies: None,
        }
    }
    pub fn preparation_requests(&self) -> ResourceRequests<'_> {
        ResourceRequests {
            system: self,
            dependencies: Some(Vec::new()),
        }
    }

    pub fn request<T: TypedResourceData>(&self, path: &Path, retention: Retention) -> Resource<T> {
        let resource = self.manager.request(path);
        if matches!(retention, Retention::Session) {
            self.retained.lock().insert(resource.clone().into_untyped());
        }
        resource
    }

    pub fn update(&self, delta: f32) {
        self.manager.state().update(delta);
    }

    pub fn register_world_loader(
        self: &Arc<Self>,
        gpu: Option<GpuContext>,
        runtime: tokio::runtime::Handle,
    ) {
        self.manager.state().loaders.set(WorldLoader {
            resources: Arc::downgrade(self),
            gpu,
            runtime,
        });
    }

    pub fn request_world(&self, name: &str) -> Resource<WorldResource> {
        self.request(&world_path(name), Retention::WhileUsed)
    }

    pub fn register_initial_world(&self, name: &str, descriptor: Arc<WorldDescriptor>) {
        let resource = Resource::new_ok(
            ResourceKind::External(world_path(name)),
            WorldResource(Some(descriptor)),
        );
        self.manager.state().push(resource.into_untyped());
    }
}

fn world_path(name: &str) -> PathBuf {
    PathBuf::from(format!("{name}.world"))
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
    gpu: Option<GpuContext>,
    runtime: tokio::runtime::Handle,
}

impl ResourceLoader for WorldLoader {
    fn extensions(&self) -> &[&str] {
        &["world"]
    }
    fn data_type_uuid(&self) -> Uuid {
        <WorldResource as TypeUuidProvider>::type_uuid()
    }
    fn load(&self, path: PathBuf, _: Arc<dyn ResourceIo>) -> BoxedLoaderFuture {
        let resources = self.resources.clone();
        let gpu = self.gpu.clone();
        let runtime = self.runtime.clone();
        Box::pin(async move {
            let resources = resources
                .upgrade()
                .ok_or_else(|| LoadError::new("Engine has shut down"))?;
            let name = path.with_extension("");
            let task = runtime.spawn(async move {
                let prepared =
                    WorldPreparation::load(name.to_str().expect("UTF-8 world key"), resources)
                        .await?;
                tokio::task::spawn_blocking(move || prepared.finish(gpu))
                    .await
                    .map_err(|e| e.to_string())
            });
            let descriptor = task
                .await
                .map_err(|e| LoadError::new(e.to_string()))?
                .map_err(LoadError::new)?;
            Ok(LoaderPayload::new(WorldResource(Some(Arc::new(
                descriptor,
            )))))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_reads_pending_ready_and_failed_resources_without_waiting() {
        use std::task::Poll;
        let handle = Resource::<WorldResource>::new_pending(ResourceKind::Embedded);
        assert!(handle.try_read(|_| ()).is_pending());
        let untyped = handle.clone().into_untyped();
        untyped.commit_ok(WorldResource::default());
        let guard = untyped.0.lock();
        assert!(handle.try_read(|_| ()).is_pending());
        drop(guard);
        assert!(matches!(handle.try_read(|_| 42), Poll::Ready(Ok(42))));

        let failed = Resource::<WorldResource>::new_pending(ResourceKind::Embedded);
        failed.clone().into_untyped().commit_error("missing map");
        assert!(matches!(failed.try_read(|_| ()), Poll::Ready(Err(_))));
    }

    #[test]
    fn session_request_keeps_the_existing_buffer_after_expiry() {
        let system = ResourceSystem::new(false);
        let asset = crate::embedded_asset!("sounds/player_explosion.wav");
        let mut preparation = system.preparation_requests();
        let first = preparation.sound(asset, Retention::WhileUsed).unwrap();
        let pinned = system.requests().sound(asset, Retention::Session).unwrap();
        assert_eq!(first.key(), pinned.key());
        let key = first.key();
        drop(first);
        drop(pinned);
        for dependency in preparation.into_dependencies() {
            pollster::block_on(dependency).unwrap();
        }
        system.update(61.0);
        assert_eq!(
            system
                .requests()
                .sound(asset, Retention::WhileUsed)
                .unwrap()
                .key(),
            key
        );
    }

    #[test]
    fn shared_map_creates_fresh_actors_and_reports_missing_maps() {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        let system = Arc::new(ResourceSystem::new(true));
        system.register_world_loader(None, runtime.handle().clone());
        let first = system.request_world("capture_the_flag_map");
        let second = system.request_world("capture_the_flag_map");
        assert_eq!(first.key(), second.key());
        let ready = runtime.block_on(first).unwrap();
        let descriptor = ready.data_ref().0.as_ref().unwrap().clone();
        let (mut actor, others) = descriptor.take_actors(&system);
        let (another, more) = descriptor.take_actors(&system);
        use crate::actor::Actor;
        actor.set_id(1234);
        assert_ne!(actor.get_id(), another.get_id());
        assert_eq!(others.len(), more.len());
        assert!(
            runtime
                .block_on(system.request_world("missing-resource-test-level"))
                .is_err()
        );
    }
}
