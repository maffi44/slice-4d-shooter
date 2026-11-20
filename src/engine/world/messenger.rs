// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use crate::{
    actor::{
        Actor,
        ActorID,
        ActorWrapper,
        Message,
    },
    engine::{
        world::physics::PhysicsSystem,
        shared_systems::EngineSharedSystems,
    },
};

use super::{
    actors::Actors,
    time::WorldTimeSystem,
    OuterWorldMessage,
    WorldContext,
};


/// Sender id of messages that come from outside the world (the engine or the world's owner).
/// Actor ids are random non-zero numbers, so 0 never collides with an actor.
pub const OUTER_WORLD_SENDER_ID: ActorID = 0;


pub struct Command
{
    pub sender: ActorID,
    pub command_type: CommandType,
}


pub enum CommandType
{
    SpawnActor(ActorWrapper),
    SpawnControlledActor(ActorID),
    RemoveActor(ActorID),
    /// An actor changes its own id; the old id is `Command::sender`.
    ChangeMyId(ActorID),
    RemoveAllHolesAndEffects,
}


/// The message bus of one World (the former `Messenger`).
///
/// Actors never mutate each other directly: they send broadcast/direct messages and commands here.
/// All delivery and command execution happens in `Messenger`; `World` only calls it.
/// Messages to the world's owner go to the outer-world mailbox.
pub struct Messenger
{
    broadcast_messages: Vec<Message>,
    direct_messages: Vec<(ActorID, Message)>,
    commands: Vec<Command>,
    outer_world_messages: Vec<OuterWorldMessage>,
}


impl Messenger
{
    pub fn new() -> Self
    {
        Messenger {
            broadcast_messages: Vec::with_capacity(100),
            direct_messages: Vec::with_capacity(100),
            commands: Vec::with_capacity(100),
            outer_world_messages: Vec::new(),
        }
    }

    #[inline]
    pub fn send_broadcast_message(&mut self, message: Message)
    {
        self.broadcast_messages.push(message);
    }

    #[inline]
    pub fn send_direct_message(&mut self, to: ActorID, message: Message)
    {
        self.direct_messages.push((to, message));
    }

    #[inline]
    pub fn send_command(&mut self, command: Command)
    {
        self.commands.push(command);
    }

    /// The queued broadcast messages, oldest first (for tests of id-change notifications).
    #[cfg(test)]
    pub(super) fn queued_broadcast_messages(&self) -> &[Message]
    {
        &self.broadcast_messages
    }

    /// A request to the world's owner (the Engine for the root world, the owner actor otherwise).
    #[inline]
    pub fn send_to_outer_world(&mut self, message: OuterWorldMessage)
    {
        self.outer_world_messages.push(message);
    }

    /// Takes one message for the world's owner (`pop`: no reallocation, the buffer keeps its capacity).
    #[inline]
    pub fn pop_outer_world_message(&mut self) -> Option<OuterWorldMessage>
    {
        self.outer_world_messages.pop()
    }

    /// Takes the messages for the world's owner one by one (each step is a `pop`, nothing is reallocated).
    /// Messages not taken (if the caller stops early) stay in the buffer.
    #[allow(dead_code)] // nested-world API (issues/programming/engine-refactoring-world-isolation/), not used by the game yet
    pub fn take_outer_world_messages(&mut self) -> impl Iterator<Item = OuterWorldMessage> + '_
    {
        std::iter::from_fn(move || self.outer_world_messages.pop())
    }


    /// The message pump: delivers broadcast and direct messages and executes commands
    /// until all buffers are empty. Order inside a buffer is `Vec::pop()` (newest first), as before:
    /// actor logic treats everything within one tick as simultaneous.
    pub fn process_messages(
        &mut self,
        actors: &mut Actors,
        physics: &PhysicsSystem,
        time: &WorldTimeSystem,
        shared: &mut EngineSharedSystems,
    )
    {
        loop
        {
            while let Some(message) = self.broadcast_messages.pop()
            {
                let mut world = WorldContext {
                    main_actor_id: actors.main_actor_id(),
                    messenger: self,
                    physics,
                    time,
                };

                for (_, actor) in actors.iter_mut()
                {
                    if actor.get_id().expect("the actor has no id") != message.from
                    {
                        actor.recieve_message(message.clone(), &mut world, shared);
                    }
                }
            }

            while let Some((to, message)) = self.direct_messages.pop()
            {
                let main_actor_id = actors.main_actor_id();

                if let Some(actor) = actors.get_mut(&to)
                {
                    let mut world = WorldContext {
                        main_actor_id,
                        messenger: self,
                        physics,
                        time,
                    };

                    actor.recieve_message(message, &mut world, shared);
                }
            }

            while let Some(command) = self.commands.pop()
            {
                self.execute_command(command, actors, physics, time, shared);
            }

            if self.direct_messages.is_empty() &&
                self.broadcast_messages.is_empty() &&
                self.commands.is_empty()
            {
                return;
            }
        }
    }


    /// Inserts actors and calls `on_added_to_world` on each.
    pub fn add_actors(
        &mut self,
        new_actors: Vec<ActorWrapper>,
        actors: &mut Actors,
        physics: &PhysicsSystem,
        time: &WorldTimeSystem,
        shared: &mut EngineSharedSystems,
    )
    {
        for actor in new_actors
        {
            self.add_actor(actor, actors, physics, time, shared);
        }
    }


    /// Like `add_actors` for one actor, and marks it as the world's main (camera) actor.
    pub fn add_main_actor(
        &mut self,
        actor: ActorWrapper,
        actors: &mut Actors,
        physics: &PhysicsSystem,
        time: &WorldTimeSystem,
        shared: &mut EngineSharedSystems,
    )
    {
        let id = self.add_actor(actor, actors, physics, time, shared);

        actors.set_main_actor_id(id);
    }


    /// Calls `on_world_started` on every actor during world activation.
    pub fn start_world(
        &mut self,
        actors: &mut Actors,
        physics: &PhysicsSystem,
        time: &WorldTimeSystem,
        shared: &mut EngineSharedSystems,
    )
    {
        let mut world = WorldContext {
            main_actor_id: actors.main_actor_id(),
            messenger: self,
            physics,
            time,
        };

        for (_, actor) in actors.iter_mut()
        {
            actor.on_world_started(&mut world, shared);
        }
    }


    fn add_actor(
        &mut self,
        actor: ActorWrapper,
        actors: &mut Actors,
        physics: &PhysicsSystem,
        time: &WorldTimeSystem,
        shared: &mut EngineSharedSystems,
    ) -> ActorID
    {
        let id = actors.insert(actor, self);

        let mut world = WorldContext {
            main_actor_id: actors.main_actor_id(),
            messenger: self,
            physics,
            time,
        };

        actors.get_mut(&id)
            .expect("the world has no actor right after adding it")
            .on_added_to_world(&mut world, shared);

        id
    }


    fn execute_command(
        &mut self,
        command: Command,
        actors: &mut Actors,
        physics: &PhysicsSystem,
        time: &WorldTimeSystem,
        shared: &mut EngineSharedSystems,
    )
    {
        match command.command_type
        {
            CommandType::SpawnActor(actor) =>
            {
                self.add_actor(actor, actors, physics, time, shared);
            }

            CommandType::SpawnControlledActor(actor_id) =>
            {
                let main_actor_id = actors.main_actor_id();

                match actors.get_mut(&actor_id)
                {
                    Some(actor) =>
                    {
                        match actor.get_actor_as_controlled_mut()
                        {
                            Some(controlled_actor) =>
                            {
                                let mut world = WorldContext {
                                    main_actor_id,
                                    messenger: self,
                                    physics,
                                    time,
                                };

                                controlled_actor.spawn(&mut world, shared);
                            }
                            None =>
                            {
                                log::warn!("SpawnControlledActor command: the given actor is not a ControlledActor");
                            }
                        }
                    }
                    None =>
                    {
                        log::warn!("SpawnControlledActor command: no actor with the given ActorID");
                    }
                }
            }

            CommandType::RemoveActor(id) =>
            {
                actors.remove(id);
            }

            CommandType::ChangeMyId(new_id) =>
            {
                actors.change_id(command.sender, new_id, self);
            }

            CommandType::RemoveAllHolesAndEffects =>
            {
                actors.retain(|_, actor| {
                    !matches!(
                        actor,
                        ActorWrapper::Hole(_) |
                        ActorWrapper::HoleGunMiss(_) |
                        ActorWrapper::MachinegunShot(_) |
                        ActorWrapper::PlayersDeathExplosion(_) |
                        ActorWrapper::HoleGunShot(_) |
                        ActorWrapper::ShootingImpact(_)
                    )
                });
            }
        }
    }
}
