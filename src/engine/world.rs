// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



pub mod static_object;
pub mod world_descriptor;
pub mod actors;
pub mod messenger;
pub mod time;
pub mod physics;
pub mod effects;

use std::sync::Arc;

use glam::UVec2;

use crate::{
    actor::{Actor, ActorID, Message, MessageType},
    engine::{
        world::physics::PhysicsSystem,
        shared_systems::EngineSharedSystems,
    },
};

use self::{
    actors::Actors,
    messenger::{Messenger, OUTER_WORLD_SENDER_ID},
    time::WorldTimeSystem,
    world_descriptor::WorldDescriptor,
};


/// One isolated simulation: its actors, message bus, physics, time and render data.
///
/// The Engine owns the root world; nested worlds are owned by actors, which decide when to
/// simulate them (`simulate(shared)` inside their own `tick`). The Engine doesn't track worlds.
pub struct World
{
    actors: Actors,
    messenger: Messenger,
    physics: PhysicsSystem,
    time: WorldTimeSystem,
    render: Option<WorldRenderSystem>,
}


pub struct WorldConfig
{
    /// `None`: the world is never rendered (also forced in headless mode).
    pub render: Option<WorldRenderConfig>,
}


#[derive(Clone, Copy, Debug)]
pub struct WorldRenderConfig
{
    /// Output texture resolution (any aspect ratio, independent of the window and the parent world).
    pub resolution: UVec2,
    /// Output texture scale factor (the texture is `resolution * scale` pixels).
    pub scale: f32,
    /// 0 = paused, 1 = every render frame, N = every Nth render frame.
    pub render_rate: u32,
}


/// The per-world part passed to actor callbacks (together with `&mut EngineSharedSystems`).
pub struct WorldContext<'a>
{
    pub messenger: &'a mut Messenger,
    pub physics: &'a PhysicsSystem,
    pub time: &'a WorldTimeSystem,
    /// The world's main (camera) actor.
    pub main_actor_id: ActorID,
}


/// Requests from a world to its owner: the Engine for the root world, the owner actor for a nested world.
/// The receiver decides what to do.
pub enum OuterWorldMessage
{
    /// Close the application. Root: the engine exits. Nested: the owner may forward it outward.
    #[allow(dead_code)] // nested-world API (issues/programming/engine-refactoring-world-isolation/), not used by the game yet
    QuitApplication,
    /// Replace the root world with a prepared one. Root: the engine does it.
    /// Nested: the owner may forward it to its own world's outer mailbox.
    ReplaceRootWorld(Arc<WorldDescriptor>),
    /// This world has finished its job; the owner may drop it. Root: ignored.
    #[allow(dead_code)] // nested-world API (issues/programming/engine-refactoring-world-isolation/), not used by the game yet
    WorldFinished,
    /// Game-specific messages for owner actors. The engine ignores them.
    #[allow(dead_code)] // nested-world API (issues/programming/engine-refactoring-world-isolation/), not used by the game yet
    Specific(SpecificOuterWorldMessage),
}


/// Hand-written, like `SpecificActorMessage`; extended as game mechanics need it.
pub enum SpecificOuterWorldMessage
{
    #[allow(dead_code)] // nested-world API (issues/programming/engine-refactoring-world-isolation/), not used by the game yet
    TutorialDemoFinished,
}


impl World
{
    /// Creates a world instance from a prepared descriptor. Cheap: the slow work (shaders,
    /// pipelines) is already in the descriptor. All per-instance creation happens here:
    /// own systems, actors, render registration, initial messages.
    pub fn new(
        descriptor: &WorldDescriptor,
        config: WorldConfig,
        shared: &mut EngineSharedSystems,
    ) -> World
    {
        let physics = PhysicsSystem::new(descriptor);

        let render = match config.render
        {
            Some(render_config) => WorldRenderSystem::new(descriptor, render_config, shared),
            None => None,
        };

        let mut world = World {
            actors: Actors::new(),
            messenger: Messenger::new(),
            physics,
            time: WorldTimeSystem::new(),
            render,
        };

        let (main_actor, actors) = descriptor.create_actors();

        world.messenger.add_main_actor(main_actor, &mut world.actors, &world.physics, &world.time, shared);
        world.messenger.add_actors(actors, &mut world.actors, &world.physics, &world.time, shared);
        world.messenger.start_world(&mut world.actors, &world.physics, &world.time, shared);
        world.messenger.process_messages(&mut world.actors, &world.physics, &world.time, shared);

        world
    }


    /// One full tick of this world: actors, messages, physics, messages, render data.
    pub fn simulate(&mut self, shared: &mut EngineSharedSystems)
    {
        self.time.begin_simulate(&shared.time);

        {
            let mut world = WorldContext {
                main_actor_id: self.actors.main_actor_id(),
                messenger: &mut self.messenger,
                physics: &self.physics,
                time: &self.time,
            };

            for (_, actor) in self.actors.iter_mut()
            {
                actor.tick(&mut world, shared);
            }
        }

        self.messenger.process_messages(&mut self.actors, &self.physics, &self.time, shared);

        self.physics.process_physics(&mut self.actors, self.time.delta(), &mut self.messenger);

        self.messenger.process_messages(&mut self.actors, &self.physics, &self.time, shared);

        if let Some(render) = &mut self.render
        {
            render.update(&self.actors, &self.time, shared);
        }
    }


    // ---- rendering control (atomics; callable any time) ----

    pub fn output_texture_id(&self) -> Option<WorldTextureId>
    {
        self.render.as_ref().map(|render| render.texture_id())
    }

    #[allow(dead_code)] // nested-world API (issues/programming/engine-refactoring-world-isolation/), not used by the game yet
    pub fn set_render_rate(&self, render_rate: u32)
    {
        if let Some(render) = &self.render
        {
            render.set_render_rate(render_rate);
        }
    }

    pub fn set_output_resolution(&self, resolution: UVec2)
    {
        if let Some(render) = &self.render
        {
            render.set_output_resolution(resolution);
        }
    }

    pub fn set_output_scale(&self, scale: f32)
    {
        if let Some(render) = &self.render
        {
            render.set_output_scale(scale);
        }
    }

    #[allow(dead_code)] // nested-world API (issues/programming/engine-refactoring-world-isolation/), not used by the game yet
    pub fn set_output_resolution_and_scale(&self, resolution: UVec2, scale: f32)
    {
        if let Some(render) = &self.render
        {
            render.set_output_resolution_and_scale(resolution, scale);
        }
    }

    pub fn output_scale(&self) -> Option<f32>
    {
        self.render.as_ref().map(|render| render.output_scale())
    }


    // ---- owner → world (sender id 0) ----

    #[allow(dead_code)] // nested-world API (issues/programming/engine-refactoring-world-isolation/), not used by the game yet
    pub fn send_broadcast_message(&mut self, message: MessageType)
    {
        self.messenger.send_broadcast_message(
            Message {
                from: OUTER_WORLD_SENDER_ID,
                remote_sender: false,
                message,
            }
        );
    }

    #[allow(dead_code)] // nested-world API (issues/programming/engine-refactoring-world-isolation/), not used by the game yet
    pub fn send_direct_message(&mut self, to: ActorID, message: MessageType)
    {
        self.messenger.send_direct_message(
            to,
            Message {
                from: OUTER_WORLD_SENDER_ID,
                remote_sender: false,
                message,
            }
        );
    }


    // ---- world → owner ----

    /// The world's requests to its owner, taken one by one (`pop`, no reallocation).
    #[allow(dead_code)] // nested-world API (issues/programming/engine-refactoring-world-isolation/), not used by the game yet
    pub fn take_outer_world_messages(&mut self) -> impl Iterator<Item = OuterWorldMessage> + '_
    {
        self.messenger.take_outer_world_messages()
    }

    /// Takes one request to the owner. Useful when the owner needs `&mut self` while handling it
    /// (the iterator would keep the world borrowed).
    pub fn pop_outer_world_message(&mut self) -> Option<OuterWorldMessage>
    {
        self.messenger.pop_outer_world_message()
    }
}


use crate::engine::shared_systems::render::{WorldRenderSystem, WorldTextureId};
