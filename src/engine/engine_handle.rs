// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2025  Timofei Molokov

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::{
    actor::{
        ActorID, ActorWrapper, Message
    },
    engine::effects::EffectType,
};

use client_server_protocol::NetCommand;

pub struct Command {
    pub sender: ActorID,
    pub command_type: CommandType,
}

pub enum CommandType {
    // SpawnProjectile(ProjectileType),
    SpawnEffect(EffectType),
    SpawnActor(ActorWrapper),
    RemoveActor(ActorID),
    NetCommand(NetCommand),
    RemoveAllHolesAndEffects,
    ShowConnectionStatusUI,
    LoadNewLevelSync(String),
    LoadNewLevelAsync(String),
}

pub struct EngineHandle {
    pub command_buffer: Vec<Command>,
    pub boardcast_message_buffer: Vec<Message>,
    pub direct_message_buffer: Vec<(ActorID, Message)>,
    // pub physics_state: PhysicsState,
}

impl EngineHandle {
    pub fn new() -> Self {
        EngineHandle {
            command_buffer: Vec::with_capacity(50),
            boardcast_message_buffer: Vec::with_capacity(50),
            direct_message_buffer: Vec::with_capacity(50),
            // physics_state: PhysicsState::new(),
        }
    }

    pub fn send_command(&mut self, command: Command) {
        self.command_buffer.push(command);
    }

    pub fn send_direct_message(&mut self, to: ActorID, message: Message) {
        self.direct_message_buffer.push((to, message));
    }

    pub fn send_boardcast_message(&mut self, message: Message) {
        self.boardcast_message_buffer.push(message);
    }
}