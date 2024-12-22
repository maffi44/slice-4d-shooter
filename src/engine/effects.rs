use glam::{Vec3, Vec4};

use crate::{
    actor::{
        wave::Wave,
        ActorWrapper
    },
    engine::engine_handle::{
        Command,
        CommandType
    }
};

use super::engine_handle::EngineHandle;


pub enum EffectType {
    DefaultPistolDecal,
}

pub struct EffectsSystem
{

}

impl EffectsSystem
{
    pub fn new() -> Self
    {
        EffectsSystem
        {

        }
    }

    pub fn spawn_wave(
        &mut self,
        engine_handle: &mut EngineHandle,
        position: Vec4,
        radii: Vec<f32>,
        colors: Vec<Vec3>,
        time_segments: Vec<f32>,
    )
    {
        assert!(
            radii.len() >= 2
        );

        assert!(
            radii.len() == colors.len() &&
            radii.len() == (time_segments.len() + 1)
        );

        let wave = Wave::new(
            position,
            radii,
            colors,
            time_segments
        );

        engine_handle.send_command(
            Command {
                sender: 0u128,
                command_type: CommandType::SpawnActor(
                    ActorWrapper::Wave(wave)
                )
            }
        );
    }
}