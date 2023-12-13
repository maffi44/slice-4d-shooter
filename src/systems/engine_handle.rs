use super::{
    projectiles::ProjectileType,
    player::PlayerID,
    player::Message,
    physics::PhysicsState,
    effects::EffectType,
};

pub struct Command {
    pub sender: PlayerID,
    pub command_type: CommandType,
}

pub enum CommandType {
    SpawnProjectile(ProjectileType),
    SendMessage(PlayerID, Message),
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