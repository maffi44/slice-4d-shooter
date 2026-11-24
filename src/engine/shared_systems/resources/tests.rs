// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use super::*;

#[test]
fn tick_reads_pending_ready_and_failed_resources_without_waiting() {
    use std::task::Poll;
    let unloaded = Resource::<WorldResource>::from(Uuid::new_v4());
    assert!(matches!(unloaded.try_read(|_| ()), Poll::Ready(Err(_))));

    let handle = Resource::<WorldResource>::new_pending(Uuid::new_v4(), ResourceKind::Embedded);
    assert!(handle.try_read(|_| ()).is_pending());
    let untyped = handle.clone().into_untyped();
    untyped.commit_ok(WorldResource::default());
    let guard = untyped.0.lock();
    assert!(handle.try_read(|_| ()).is_pending());
    drop(guard);
    assert!(matches!(handle.try_read(|_| 42), Poll::Ready(Ok(42))));

    let failed = Resource::<WorldResource>::new_pending(Uuid::new_v4(), ResourceKind::Embedded);
    failed.clone().into_untyped().commit_error(PathBuf::new(), "missing map");
    assert!(matches!(failed.try_read(|_| ()), Poll::Ready(Err(_))));
}

#[test]
fn session_request_keeps_the_existing_buffer_after_expiry() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let system = ResourceSystem::new(runtime.handle().clone(), Some(watch::channel(None).1));
    let asset = crate::embedded_asset!("sounds/player_explosion.wav");
    let mut dependencies = Vec::new();
    let mut preparation = system.loader_with_dependencies(&mut dependencies);
    let first = preparation
        .load_media::<SoundBuffer>(asset, Retention::WhileUsed)
        .unwrap();
    let pinned = system
        .loader()
        .load::<SoundBuffer>(asset, Retention::Session);
    assert_eq!(first.key(), pinned.key());
    let key = first.key();
    drop(first);
    drop(pinned);
    for dependency in dependencies {
        runtime.block_on(dependency).unwrap();
    }
    system.update(61.0);
    assert_eq!(
        system
            .loader()
            .load_media::<SoundBuffer>(asset, Retention::WhileUsed)
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
    let system = ResourceSystem::new(runtime.handle().clone(), None);
    let first = system
        .loader()
        .load::<WorldResource>("capture_the_flag_map", Retention::WhileUsed);
    let second = system
        .loader()
        .load::<WorldResource>("./capture_the_flag_map.json", Retention::WhileUsed);
    assert_eq!(first.key(), second.key());
    let ready = runtime.block_on(first).unwrap();
    let descriptor = ready.data_ref().0.as_ref().unwrap().clone();
    let (mut actor, others) = descriptor.take_actors(&mut system.loader());
    let (another, more) = descriptor.take_actors(&mut system.loader());
    use crate::actor::Actor;
    actor.set_id(1234);
    assert_ne!(actor.get_id(), another.get_id());
    assert_eq!(others.len(), more.len());
    let invalid = system
        .loader()
        .load::<WorldResource>("", Retention::WhileUsed);
    assert!(matches!(
        invalid.try_read(|_| ()),
        std::task::Poll::Ready(Err(_))
    ));
    assert!(runtime
        .block_on(
            system
                .loader()
                .load::<WorldResource>("missing-resource-test-level", Retention::WhileUsed)
        )
        .is_err());
}

#[test]
fn headless_skips_media_but_allows_required_sound_data() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let system = ResourceSystem::new(runtime.handle().clone(), None);
    let mut dependencies = Vec::new();
    let mut scope = system.loader_with_dependencies(&mut dependencies);
    let key = crate::embedded_asset!("sounds/shotgun_shot.wav");
    assert!(scope
        .load_media::<SoundBuffer>(key, Retention::Session)
        .is_none());
    assert!(dependencies.is_empty());
    assert!(system.retained.lock().is_empty());
    assert!(system.manager.state().is_empty());
    runtime
        .block_on(
            system
                .loader()
                .load::<SoundBuffer>(key, Retention::WhileUsed),
        )
        .unwrap();
}

#[test]
fn both_preparations_observe_a_shared_sound_failure() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let system = ResourceSystem::new(runtime.handle().clone(), Some(watch::channel(None).1));
    let path = std::env::temp_dir().join(format!("{}.wav", Uuid::new_v4()));
    std::fs::write(&path, b"corrupt WAV").unwrap();
    let mut a_dependencies = Vec::new();
    let mut b_dependencies = Vec::new();
    let mut a = system.loader_with_dependencies(&mut a_dependencies);
    let mut b = system.loader_with_dependencies(&mut b_dependencies);
    let first = a
        .load_media::<SoundBuffer>(path.to_str().unwrap(), Retention::WhileUsed)
        .unwrap();
    let second = b.load::<SoundBuffer>(path.to_str().unwrap(), Retention::WhileUsed);
    assert_eq!(first.key(), second.key());
    for deps in [a_dependencies, b_dependencies] {
        assert_eq!(deps.len(), 1);
        let error = runtime
            .block_on(deps.into_iter().next().unwrap())
            .unwrap_err();
        assert!(format!("{error:?}").contains("Invalid sound"));
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn world_load_fails_when_graphics_startup_ends_without_a_device() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    let (sender, receiver) = watch::channel(None);
    let system = ResourceSystem::new(runtime.handle().clone(), Some(receiver));
    let world = system
        .loader()
        .load::<WorldResource>("capture_the_flag_map", Retention::WhileUsed);
    drop(sender);
    let result = runtime.block_on(async {
        tokio::time::timeout(std::time::Duration::from_secs(10), world)
            .await
            .unwrap()
    });
    assert!(format!("{:?}", result.unwrap_err()).contains("GPU initialization ended"));
}

#[test]
fn required_gpu_asset_has_a_terminal_headless_error() {
    #[derive(Debug, Default, Reflect, Visit)]
    #[reflect(non_cloneable)]
    struct TestTexture {}

    impl TypeUuidProvider for TestTexture {
        fn type_uuid() -> Uuid {
            uuid!("2fbd3e17-e49b-4c67-965a-4e49630c175e")
        }
    }
    impl ResourceData for TestTexture {
        fn type_uuid(&self) -> Uuid {
            <Self as TypeUuidProvider>::type_uuid()
        }
        fn can_be_saved(&self) -> bool {
            false
        }
        fn save(&mut self, _: &Path) -> Result<(), Box<dyn Error>> {
            unreachable!()
        }
        fn try_clone_box(&self) -> Option<Box<dyn ResourceData>> {
            None
        }
    }
    impl Asset for TestTexture {
        const REQUIRES_GPU: bool = true;
        fn manager_path(key: &str) -> Result<Cow<'_, Path>, String> {
            Ok(Cow::Borrowed(Path::new(key)))
        }
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let system = ResourceSystem::new(runtime.handle().clone(), None);
    let mut dependencies = Vec::new();
    let mut preparation = system.loader_with_dependencies(&mut dependencies);
    let texture = preparation.load::<TestTexture>("hud.png", Retention::Session);
    let std::task::Poll::Ready(Err(error)) = texture.try_read(|_| ()) else {
        panic!("A required GPU asset must fail immediately in headless mode");
    };
    assert!(format!("{error:?}").contains("requires a GPU"));
    assert_eq!(dependencies.len(), 1);
    assert!(system.manager.state().is_empty());
}
