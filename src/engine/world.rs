pub mod static_object;
pub mod level;

use self::level::Level;

use crate::{
    actor::{
        player::{
            player_settings::PlayerSettings,
            PlayerMessages
        },
        Actor,
        ActorID,
        ActorWrapper,
        Message,
        MessageType,
        SpecificActorMessage,
    },
    engine::{
        engine_handle::{
            CommandType,
            EngineHandle,
        },
        physics::PhysicsSystem
    },
};

use core::panic;
use std::collections::HashMap;

use super::{audio::AudioSystem, engine_handle::Command, net::{NetCommand, NetSystem}, ui::UISystem};

pub struct World {
    pub level: Level,
    pub actors: HashMap<ActorID, ActorWrapper>,
    pub main_player_id: ActorID,
    pub players_settings: PlayerSettings,
}

impl World {

    pub async fn new(engine_handle: &mut EngineHandle, players_settings: PlayerSettings) -> Self {
        
        // 0 it is id of engine
        // in case when engine send message to the some actor
        // sender property will be 0      
        let (level, actors) = Level::load_level().await;

        log::info!("world system: level downloaded and init");

        let mut world = World {
            actors: HashMap::with_capacity(actors.len()),
            players_settings,
            level,
            main_player_id: 0,
        };

        for actor in actors {
            world.add_actor_to_world(actor, engine_handle);
        }

        world
    }

    pub fn send_messages_and_process_commands(
        &mut self,
        net_system: &mut NetSystem,
        physics_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        engine_handle: &mut EngineHandle
    ) {
        
        loop {
                while let Some(message) = engine_handle.boardcast_message_buffer.pop() {
                    self.send_boardcast_messages(message, engine_handle, physics_system, audio_system, ui_system)                
                }

                while let Some((to, message)) = engine_handle.direct_message_buffer.pop() {
                    self.send_direct_messages(to, message, engine_handle, physics_system, audio_system, ui_system)                
                }

                while let Some(command) = engine_handle.command_buffer.pop() {
                    self.execute_command(command, net_system, physics_system, engine_handle, audio_system, ui_system);
                }

                if engine_handle.direct_message_buffer.is_empty() &&
                    engine_handle.boardcast_message_buffer.is_empty() &&
                    engine_handle.command_buffer.is_empty()
                {   
                    return;
                }
            }
    }

    fn execute_command(
        &mut self,
        command: Command,
        net_system: &mut NetSystem,
        physics_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
    ) {
        let from = command.sender;

        match command.command_type {
            CommandType::SpawnEffect(_) => {}
            CommandType::SpawnActor(actor) => {
                self.add_actor_to_world(actor, engine_handle);
            }
            CommandType::RemoveActor(id) => {
                self.remove_actor_from_world(id);
            }
            CommandType::RespawnPlayer(id) => {
            
                let spawn_position = self.level.get_random_spawn_position();

                if let Some(player) = self.actors.get_mut(&id) {
                    
                    if let ActorWrapper::Player(player) = player {
                        player.respawn(
                            spawn_position,
                            physics_system,
                            ui_system,
                            audio_system,
                            engine_handle
                        );
                    } else {
                        panic!("Player send wrong ID into RespawnPlayer command. Actor with this ID is not player")
                    }
                } else {
                    //this case is possible when player send command to get respawn and before
                    //command is processed player's id was changed (for example when game connected to singnaling server)

                    // temporal solution is get main_player_id (todo: need to detect when actor change ID, store pair (new 
                    // and old ids) for one frame and change old id for new in this case)

                    let player = self.actors.get_mut(&self.main_player_id).expect("World have not actor with main_player_id");
                    
                    if let ActorWrapper::Player(player) = player {
                        
                        player.respawn(
                            spawn_position,
                            physics_system,
                            ui_system,
                            audio_system,
                            engine_handle
                        );

                    } else {
                        panic!("Actor with main_player_id is not Player");
                    }
                }

            }
            CommandType::NetCommand(command) => {
                match command {
                    NetCommand::NetSystemIsConnectedAndGetNewPeerID(new_id) => {
                        self.change_actor_id(self.main_player_id, new_id, engine_handle);

                        self.main_player_id = new_id;                       
                    },
                    NetCommand::SendBoardcastNetMessageReliable(message) => {
                        net_system.send_boardcast_message_reliable(message);
                    },

                    NetCommand::SendBoardcastNetMessageUnreliable(message) => {
                        net_system.send_boardcast_message_unreliable(message);
                    },

                    NetCommand::SendDirectNetMessageReliable(message, peer) => {
                        net_system.send_direct_message_reliable(message, peer);
                    },

                    NetCommand::SendDirectNetMessageUnreliable(message, peer) => {
                        net_system.send_direct_message_unreliable(message, peer);
                    },

                    NetCommand::PeerConnected(peer_id) => {
                        engine_handle.send_boardcast_message(
                            Message {
                                from: 0u128,
                                message: MessageType::SpecificActorMessage(
                                    SpecificActorMessage::PLayerMessages(
                                        PlayerMessages::NewPeerConnected(peer_id)
                                    )
                                )
                            }
                        )
                    },

                    NetCommand::PeerDisconnected(id) => {
                        self.remove_actor_from_world(id.0.as_u128());
                    }
                }
            }
        }
    }

    fn send_direct_messages(
        &mut self,
        to: ActorID,
        message: Message,
        engine_handle: &mut EngineHandle,
        physics_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
    ) {
        if let Some(actor) = self.actors.get_mut(&to) {
            actor.recieve_message(&message, engine_handle, physics_system, audio_system, ui_system);
        }
    }

    fn send_boardcast_messages(
        &mut self,
        message: Message,
        engine_handle: &mut EngineHandle,
        physics_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
    ) {
        for (_, actor) in self.actors.iter_mut() {
            if actor.get_id().expect("actor does not have id") != message.from {
                actor.recieve_message(&message, engine_handle, physics_system, audio_system, ui_system);
            } 
        }
    }

    fn change_actor_id(&mut self, old_id: ActorID, new_id: ActorID, engine_handle: &mut EngineHandle) {
        if let Some(mut actor) = self.remove_actor_from_world(old_id) {
            actor.set_id(new_id, engine_handle);

            if let Some(mut swaped_actor) = self.actors.insert(new_id, actor) {
                
                let new_id_for_swaped_actor = self.get_new_random_uniq_id();

                swaped_actor.set_id(new_id_for_swaped_actor, engine_handle);

                self.actors.insert(new_id_for_swaped_actor, swaped_actor);
            }
        }
    }

    pub fn add_actor_to_world(&mut self, mut actor: ActorWrapper, engine_handle: &mut EngineHandle) -> ActorID {

        let id = match actor.get_id() {
            Some(id) => id,
            None => {
                let new_id = self.get_new_random_uniq_id();

                actor.init(new_id);

                new_id
            },
        };

        if let Some(mut swaped_actor) = self.actors.insert(id, actor) {
                
            let new_id_for_swaped_actor = self.get_new_random_uniq_id();

            swaped_actor.set_id(new_id_for_swaped_actor, engine_handle);

            self.actors.insert(new_id_for_swaped_actor, swaped_actor);
        }

        id
    }

    pub fn remove_actor_from_world(&mut self, id: ActorID) -> Option<ActorWrapper> {

        self.actors.remove(&id)
    }

    pub fn tick(
        &mut self,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        delta: f32
    ) {
        for (_, actor) in self.actors.iter_mut() {
            actor.tick(
                physic_system,
                engine_handle,
                audio_system,
                ui_system,
                delta
            )
        }
    }

    fn get_new_random_uniq_id(&self) -> ActorID {
        let mut new_id = get_random_non_zero_id();

        while self.actors.contains_key(&new_id) {
            new_id = get_random_non_zero_id();
        }

        new_id
    }

}

fn get_random_non_zero_id() -> ActorID {
    let mut bytes : [u8;16] = [0;16];
    let res = getrandom::getrandom(&mut bytes);
    
    if let Err(err) = res {
        panic!("Can't make random u128 in get_random_id function");
    }

    let mut id: u128 = u128::from_be_bytes(bytes);

    // 0 it is reserved ID for the Engine itself
    if id == 0u128 {
        id = get_random_non_zero_id();
    }

    id
}