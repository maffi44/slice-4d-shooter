use crate::{
    actor::{
        ActorID, ActorWrapper, Message
    },
    engine::effects::EffectType,
};



pub struct Command {
    pub sender: ActorID,
    pub command_type: CommandType,
}

pub enum CommandType {
    // SpawnProjectile(ProjectileType),
    SpawnEffect(EffectType),
    SpawnActor(ActorWrapper),
    RemoveActor(ActorID),
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