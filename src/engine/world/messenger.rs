// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use std::collections::VecDeque;

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
    /// Boxed: an `ActorWrapper` is as large as the largest actor (about 1.5 KB), and every slot of the
    /// message queue has the size of its largest item. One allocation per spawn keeps queue slots
    /// message-sized; spawning already allocates in most actors' constructors.
    SpawnActor(Box<ActorWrapper>),
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
///
/// Ordering: broadcast messages, direct messages and commands are delivered/executed in one FIFO
/// order, the order in which they were sent (see `process_messages`). They share one `VecDeque`,
/// so the pump reads a single sequential buffer; all buffers keep their capacity between ticks.
pub struct Messenger
{
    queue: VecDeque<Queued>,
    outer_world_messages: VecDeque<OuterWorldMessage>,
}


/// One item of the message pump. Its size is about that of `Message` plus a recipient id:
/// the large spawn payload is boxed in `CommandType::SpawnActor`.
enum Queued
{
    Broadcast(Message),
    Direct(ActorID, Message),
    Command(Command),
}


impl Messenger
{
    pub fn new() -> Self
    {
        Messenger {
            queue: VecDeque::with_capacity(300),
            outer_world_messages: VecDeque::new(),
        }
    }

    #[inline]
    pub fn send_broadcast_message(&mut self, message: Message)
    {
        self.queue.push_back(Queued::Broadcast(message));
    }

    #[inline]
    pub fn send_direct_message(&mut self, to: ActorID, message: Message)
    {
        self.queue.push_back(Queued::Direct(to, message));
    }

    #[inline]
    pub fn send_command(&mut self, command: Command)
    {
        self.queue.push_back(Queued::Command(command));
    }

    /// The queued broadcast messages, oldest first (for tests of actor notifications).
    #[cfg(test)]
    pub(crate) fn queued_broadcast_messages(&self) -> impl Iterator<Item = &Message>
    {
        self.queue.iter().filter_map(|item| match item
        {
            Queued::Broadcast(message) => Some(message),
            _ => None,
        })
    }

    /// The queued direct messages with their recipients, oldest first (for tests of actor requests).
    #[cfg(test)]
    pub(crate) fn queued_direct_messages(&self) -> impl Iterator<Item = (ActorID, &Message)>
    {
        self.queue.iter().filter_map(|item| match item
        {
            Queued::Direct(to, message) => Some((*to, message)),
            _ => None,
        })
    }

    /// A request to the world's owner (the Engine for the root world, the owner actor otherwise).
    #[inline]
    pub fn send_to_outer_world(&mut self, message: OuterWorldMessage)
    {
        self.outer_world_messages.push_back(message);
    }

    /// Takes the oldest message for the world's owner (FIFO; the buffer keeps its capacity).
    #[inline]
    pub fn pop_outer_world_message(&mut self) -> Option<OuterWorldMessage>
    {
        self.outer_world_messages.pop_front()
    }

    /// Takes the messages for the world's owner one by one, oldest first (nothing is reallocated).
    /// Messages not taken (if the caller stops early) stay in the buffer.
    #[allow(dead_code)] // nested-world API (issues/programming/engine-refactoring-world-isolation/), not used by the game yet
    pub fn take_outer_world_messages(&mut self) -> impl Iterator<Item = OuterWorldMessage> + '_
    {
        std::iter::from_fn(move || self.outer_world_messages.pop_front())
    }


    /// The message pump: delivers broadcast and direct messages and executes commands one by one,
    /// in the order they were sent, until nothing is queued. Items sent while an item is handled are
    /// appended and handled after everything queued before them (breadth-first).
    ///
    /// Consequences callers rely on:
    /// - successive state updates are applied in sending order, so the last one sent wins
    ///   (network packets are queued in arrival order by `NetSession`);
    /// - lifecycle commands are ordered with messages: a direct message sent after `SpawnActor` reaches
    ///   the spawned actor, one sent after `RemoveActor` is dropped; `RemoveActor` followed by
    ///   `SpawnActor` with the same id leaves the new actor;
    /// - ids are resolved when an item is handled, not when it is sent.
    pub fn process_messages(
        &mut self,
        actors: &mut Actors,
        physics: &PhysicsSystem,
        time: &WorldTimeSystem,
        render: &mut Option<super::WorldRenderSystem>,
        ui: &mut super::UISystem,
        shared: &mut EngineSharedSystems,
    )
    {
        while let Some(item) = self.queue.pop_front()
        {
            match item
            {
                Queued::Broadcast(message) =>
                {
                    let mut world = WorldContext {
                        main_actor_id: actors.main_actor_id(),
                        messenger: self,
                        render: render.as_mut(),
                        ui,
                        physics,
                        time,
                    };

                    for (_, actor) in actors.iter_mut()
                    {
                        if actor.get_id().expect("the actor has no id") != message.from
                        {
                            actor.recieve_message(&message, &mut world, shared);
                        }
                    }
                }

                Queued::Direct(to, message) =>
                {
                    let main_actor_id = actors.main_actor_id();

                    if let Some(actor) = actors.get_mut(&to)
                    {
                        let mut world = WorldContext {
                            main_actor_id,
                            messenger: self,
                            render: render.as_mut(),
                            ui,
                            physics,
                            time,
                        };

                        actor.recieve_message(&message, &mut world, shared);
                    }
                }

                Queued::Command(command) =>
                {
                    self.execute_command(command, actors, physics, time, render, ui, shared);
                }
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
        render: &mut Option<super::WorldRenderSystem>,
        ui: &mut super::UISystem,
        shared: &mut EngineSharedSystems,
    )
    {
        for actor in new_actors
        {
            self.add_actor(actor, actors, physics, time, render, ui, shared);
        }
    }


    /// Like `add_actors` for one actor, and marks it as the world's main (camera) actor.
    pub fn add_main_actor(
        &mut self,
        actor: ActorWrapper,
        actors: &mut Actors,
        physics: &PhysicsSystem,
        time: &WorldTimeSystem,
        render: &mut Option<super::WorldRenderSystem>,
        ui: &mut super::UISystem,
        shared: &mut EngineSharedSystems,
    )
    {
        let id = self.add_actor(actor, actors, physics, time, render, ui, shared);

        actors.set_main_actor_id(id);
    }


    /// Calls `on_world_started` on every actor during world activation.
    pub fn start_world(
        &mut self,
        actors: &mut Actors,
        physics: &PhysicsSystem,
        time: &WorldTimeSystem,
        render: &mut Option<super::WorldRenderSystem>,
        ui: &mut super::UISystem,
        shared: &mut EngineSharedSystems,
    )
    {
        let mut world = WorldContext {
            main_actor_id: actors.main_actor_id(),
            messenger: self,
            render: render.as_mut(),
            ui,
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
        render: &mut Option<super::WorldRenderSystem>,
        ui: &mut super::UISystem,
        shared: &mut EngineSharedSystems,
    ) -> ActorID
    {
        let id = actors.insert(actor, self);

        let mut world = WorldContext {
            main_actor_id: actors.main_actor_id(),
            messenger: self,
            render: render.as_mut(),
            ui,
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
        render: &mut Option<super::WorldRenderSystem>,
        ui: &mut super::UISystem,
        shared: &mut EngineSharedSystems,
    )
    {
        match command.command_type
        {
            CommandType::SpawnActor(actor) =>
            {
                self.add_actor(*actor, actors, physics, time, render, ui, shared);
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
                                    render: render.as_mut(),
                                    ui,
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


#[cfg(test)]
mod tests
{
    use super::*;
    use crate::actor::{CommonActorsMessage, MessageType};

    fn message(from: ActorID) -> Message
    {
        Message {
            from,
            remote_sender: false,
            message: MessageType::CommonActorsMessages(CommonActorsMessage::IWasChangedMyId(from)),
        }
    }

    /// `(kind, id)` of an item: the sender for messages, the removed actor for commands.
    fn describe(item: Queued) -> (&'static str, ActorID)
    {
        match item
        {
            Queued::Broadcast(message) => ("broadcast", message.from),
            Queued::Direct(to, _) => ("direct", to),
            Queued::Command(Command { command_type: CommandType::RemoveActor(id), .. }) => ("command", id),
            Queued::Command(_) => ("other command", 0),
        }
    }

    fn remove(id: ActorID) -> Command
    {
        Command { sender: OUTER_WORLD_SENDER_ID, command_type: CommandType::RemoveActor(id) }
    }

    #[test]
    fn items_of_all_kinds_are_taken_in_sending_order()
    {
        let mut messenger = Messenger::new();

        messenger.send_broadcast_message(message(1));
        messenger.send_command(remove(2));
        messenger.send_direct_message(3, message(30));
        messenger.send_broadcast_message(message(4));
        messenger.send_command(remove(5));

        let mut taken = Vec::new();
        while let Some(item) = messenger.queue.pop_front()
        {
            taken.push(describe(item));

            // Items sent while handling are appended after everything already queued.
            if taken.len() == 1
            {
                messenger.send_direct_message(6, message(60));
            }
        }

        assert_eq!(taken, vec![
            ("broadcast", 1),
            ("command", 2),
            ("direct", 3),
            ("broadcast", 4),
            ("command", 5),
            ("direct", 6),
        ]);
    }

    /// Guards against an inline large payload (e.g. an unboxed `ActorWrapper`) making every queue slot large.
    #[test]
    fn queue_items_stay_message_sized()
    {
        let message = std::mem::size_of::<Message>();
        let item = std::mem::size_of::<Queued>();

        eprintln!("Message: {message} bytes, Command: {} bytes, queue item: {item} bytes", std::mem::size_of::<Command>());

        assert!(item <= message + 2 * std::mem::size_of::<ActorID>());
    }

    #[test]
    fn outer_world_messages_are_taken_in_sending_order()
    {
        let mut messenger = Messenger::new();

        messenger.send_to_outer_world(OuterWorldMessage::QuitApplication);
        messenger.send_to_outer_world(OuterWorldMessage::WorldFinished);

        assert!(matches!(messenger.pop_outer_world_message(), Some(OuterWorldMessage::QuitApplication)));
        assert!(matches!(messenger.take_outer_world_messages().next(), Some(OuterWorldMessage::WorldFinished)));
        assert!(messenger.pop_outer_world_message().is_none());
    }
}
