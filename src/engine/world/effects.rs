// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use glam::{Vec3, Vec4};

use crate::{
    actor::{
        wave::Wave,
        ActorWrapper
    },
    engine::world::messenger::{
        Command,
        CommandType,
        Messenger,
    }
};


/// Spawns a visual wave actor (former `EffectsSystem::spawn_wave`; `EffectsSystem` was an empty struct).
pub fn spawn_wave(
    messenger: &mut Messenger,
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

    messenger.send_command(
        Command {
            sender: 0u128,
            command_type: CommandType::SpawnActor(
                Box::new(ActorWrapper::Wave(wave))
            )
        }
    );
}
