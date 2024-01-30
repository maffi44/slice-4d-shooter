use super::{
    projectiles::ProjectileType,
    actor::{
        Message,
        ActorID,
    },
    physics::PhysicsState,
    effects::EffectType,
};

pub struct Command {
    pub sender: ActorID,
    pub command_type: CommandType,
}

pub enum CommandType {
    SpawnProjectile(ProjectileType),
    SendMessage(ActorID, Message),
    SpawnEffect(EffectType),
}

pub struct EngineHandle {
    pub command_buffer: Vec<Command>,
    pub physics_state: PhysicsState,
}

impl EngineHandle {
    pub fn new() -> Self {
        EngineHandle {
            command_buffer: Vec::with_capacity(50),
            physics_state: PhysicsState::new(),
        }
    }

    pub fn send_command(&mut self, command: Command) {
        self.command_buffer.push(command);
    }
}