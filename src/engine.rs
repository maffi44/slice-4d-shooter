// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



pub mod shared_systems;
pub mod world;
pub mod transform;
pub mod main_loop;
mod loader;

pub use loader::{load, EngineLoader, LoadedEngine};

use std::sync::Arc;

use glam::UVec2;
use winit::{dpi::PhysicalSize, window::Window};

use self::{
    shared_systems::{resources::ResourceSystem, EngineSharedSystems},
    world::{
        world_descriptor::WorldDescriptor,
        OuterWorldMessage,
        World,
        WorldConfig,
        WorldRenderConfig,
    },
};


/// Output scale steps of the `i`/`O` keys (as before: 5% steps within 0.1..1.0).
const OUTPUT_SCALE_STEP: f32 = 0.05;
const MIN_OUTPUT_SCALE: f32 = 0.1;
const MAX_OUTPUT_SCALE: f32 = 1.0;


/// The engine: the shared systems and the root world.
///
/// The engine knows only the root world. Nested worlds are owned and simulated by actors.
///
/// Created only after EngineLoader has finished. MainLoop owns event delivery and tick scheduling.
pub struct Engine
{
    // Drop worlds and shared systems (including the render thread) before the window.
    root_world: Option<World>,
    resource_system: Arc<ResourceSystem>,
    pub shared: EngineSharedSystems,
    window: Option<Arc<Window>>,
    quit_requested: bool,
}


impl Engine
{
    /// Consumes a completed load. Only per-instance world activation remains on this thread.
    pub fn new(loaded: LoadedEngine, window: Option<Arc<Window>>) -> Engine
    {
        let mut engine = Engine {
            root_world: None,
            resource_system: loaded.resource_system,
            shared: loaded.shared,
            window,
            quit_requested: false,
        };
        engine.set_root_world(loaded.descriptor);
        // Loading time must not become the first simulation delta.
        engine.shared.time.init();
        log::info!("engine: ready");
        engine
    }


    pub fn window(&self) -> Option<&Arc<Window>>
    {
        self.window.as_ref()
    }


    /// Manual maintenance for a future long-running engine if registry growth becomes measurable.
    /// Returns the number of path mappings removed for resources already evicted by Fyrox.
    #[allow(dead_code)] // Reserved for explicit maintenance if needed.
    pub fn prune_resource_registry(&self) -> usize
    {
        self.resource_system.prune_registry()
    }


    /// One engine tick.
    pub fn tick(&mut self)
    {
        self.shared.pre_simulate();
        self.resource_system.update(self.shared.time.prev_frame_duration());

        if let Some(root_world) = &mut self.root_world
        {
            root_world.simulate(&mut self.shared);
        }

        // One message at a time: handling one needs `&mut self` (e.g. replacing the root world),
        // so the world can't stay borrowed by an iterator. Oldest first; nothing is reallocated.
        while let Some(message) = self.root_world.as_mut().and_then(|world| world.pop_outer_world_message())
        {
            self.handle_outer_world_message(message);
        }

        self.process_render_hotkeys();

        self.shared.post_simulate();
    }


    pub fn quit_requested(&self) -> bool
    {
        self.quit_requested
    }


    pub fn on_window_resized(&mut self, size: PhysicalSize<u32>)
    {
        if let Some(render) = &mut self.shared.render
        {
            render.resize_surface(size);
        }

        if size.width > 0 && size.height > 0
        {
            if let Some(root_world) = &self.root_world
            {
                root_world.set_output_resolution(UVec2::new(size.width, size.height));
            }
        }
    }


    /// Replaces the root world (also used for the initial world). Activates actors and world resources.
    fn set_root_world(&mut self, descriptor: Arc<WorldDescriptor>)
    {
        let render_config = self.window.as_ref().map(|window| {
            let size = window.inner_size();

            WorldRenderConfig {
                resolution: UVec2::new(size.width.max(1), size.height.max(1)),
                scale: self.shared.global_settings.user.screen_resolution_scale
                    .clamp(MIN_OUTPUT_SCALE, MAX_OUTPUT_SCALE),
                render_rate: 1,
            }
        });

        let new_world = descriptor.give_me_world(
            WorldConfig { render: render_config },
            &mut self.shared,
        );

        if let Some(render) = &self.shared.render
        {
            render.present_world(new_world.output_texture_id());
        }

        // the old world (if any) is dropped here: its actors, nested worlds and render slot
        let _old_world = self.root_world.replace(new_world);

        log::info!("engine: root world is '{}'", descriptor.name);
    }


    fn handle_outer_world_message(&mut self, message: OuterWorldMessage)
    {
        match message
        {
            OuterWorldMessage::QuitApplication =>
            {
                self.quit_requested = true;
            }
            OuterWorldMessage::ReplaceRootWorld(descriptor) =>
            {
                self.set_root_world(descriptor);
            }
            // Only `QuitApplication` exits; a finished root world keeps running
            // (issues/programming/root-world-finished/).
            OuterWorldMessage::WorldFinished => {}
            OuterWorldMessage::Specific(_) => {}
        }
    }


    /// `i`/`O`: root world output scale, `P`: shadows (input actions; persisted in user settings).
    fn process_render_hotkeys(&mut self)
    {
        let actions = self.shared.input.actions();

        let increase = actions.increase_render_quality.is_action_just_pressed();
        let decrease = actions.decrease_render_quality.is_action_just_pressed();
        let toggle_shadows = actions.shadows_toggle.is_action_just_pressed();

        if increase || decrease
        {
            if let Some(root_world) = &self.root_world
            {
                if let Some(scale) = root_world.output_scale()
                {
                    let new_scale = if increase
                    {
                        if scale > 0.995 {scale} else {(scale + OUTPUT_SCALE_STEP).clamp(MIN_OUTPUT_SCALE, MAX_OUTPUT_SCALE)}
                    }
                    else
                    {
                        if scale < 0.105 {scale} else {(scale - OUTPUT_SCALE_STEP).clamp(MIN_OUTPUT_SCALE, MAX_OUTPUT_SCALE)}
                    };

                    root_world.set_output_scale(new_scale);

                    self.shared.global_settings.user.screen_resolution_scale = new_scale;
                    self.shared.global_settings.save();
                }
            }
        }

        if toggle_shadows
        {
            let user_settings = &mut self.shared.global_settings.user;
            user_settings.shadows_enabled = !user_settings.shadows_enabled;

            self.shared.global_settings.save();
        }
    }
}


#[cfg(test)]
pub(crate) mod tests
{
    use super::*;

    fn loaded_headless_map() -> LoadedEngine
    {
        use shared_systems::resources::{Retention, WorldResource};

        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let resource_system = ResourceSystem::new(runtime.handle().clone(), None);
        let mut resources = resource_system.loader();
        let map = resources.load::<WorldResource>("capture_the_flag_map", Retention::WhileUsed);
        let descriptor = runtime.block_on(map).unwrap().data_ref().0.as_ref().unwrap().clone();
        let shared = EngineSharedSystems::new(true, runtime, Default::default(), resources);
        LoadedEngine { resource_system, shared, descriptor }
    }

    pub(crate) fn headless_map() -> (EngineSharedSystems, Arc<WorldDescriptor>)
    {
        let loaded = loaded_headless_map();
        (loaded.shared, loaded.descriptor)
    }

    pub(crate) fn headless_loader(runtime: &tokio::runtime::Runtime) -> shared_systems::resources::ResourceLoader<'static>
    {
        ResourceSystem::new(runtime.handle().clone(), None).loader()
    }

    #[test]
    fn returning_to_a_world_respects_source_and_retention_after_expiry()
    {
        use shared_systems::resources::{Retention, WorldResource};

        let loaded = loaded_headless_map();
        let initial = loaded.descriptor.clone();
        let mut engine = Engine::new(loaded, None);

        let file = std::env::temp_dir().join(format!("{}.json", fyrox_core::uuid::Uuid::new_v4()));
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/assets/maps/capture_the_flag_map.json");
        std::fs::copy(fixture, &file).unwrap();
        let game_key = file.to_str().unwrap();
        let game = engine.shared.resources.load::<WorldResource>(game_key, Retention::WhileUsed);
        let game = engine.shared.runtime.block_on(game).unwrap();
        let game_uuid = game.resource_uuid();
        engine.set_root_world(game.data_ref().0.as_ref().unwrap().clone());

        // Re-enter the initial map after evicting it. Both CLI and marked spellings select
        // the same source, even when a developer has a local override in the working directory.
        engine.resource_system.update(61.0);
        let menu_key = crate::embedded_asset!("maps/capture_the_flag_map.json");
        let menu = engine.shared.resources.load::<WorldResource>(menu_key, Retention::Session);
        let menu = engine.shared.runtime.block_on(menu).unwrap().data_ref().0.as_ref().unwrap().clone();
        assert!(!Arc::ptr_eq(&initial, &menu));
        assert_eq!(initial.name, menu.name);
        engine.set_root_world(menu.clone());

        // A session pin survives the same round trip with all caller handles released.
        engine.set_root_world(game.data_ref().0.as_ref().unwrap().clone());
        drop(game);
        std::fs::remove_file(&file).unwrap();
        engine.resource_system.update(61.0);
        assert!(engine.prune_resource_registry() > 0);
        assert_eq!(engine.prune_resource_registry(), 0);
        let returning = engine.shared.resources.load::<WorldResource>("capture_the_flag_map", Retention::WhileUsed);
        let descriptor = returning.data_ref().0.as_ref().unwrap().clone();
        assert!(Arc::ptr_eq(&menu, &descriptor));
        engine.set_root_world(descriptor);
        let expired = engine.shared.resources.load::<WorldResource>(game_key, Retention::WhileUsed);
        assert_ne!(expired.resource_uuid(), game_uuid);
        assert!(engine.shared.runtime.block_on(expired).is_err());
    }

    #[test]
    fn root_world_finished_keeps_running_and_quit_application_exits()
    {
        let mut engine = Engine::new(loaded_headless_map(), None);

        engine.handle_outer_world_message(OuterWorldMessage::WorldFinished);
        assert!(!engine.quit_requested());
        assert!(engine.root_world.is_some());

        engine.handle_outer_world_message(OuterWorldMessage::QuitApplication);
        assert!(engine.quit_requested());
    }
}
