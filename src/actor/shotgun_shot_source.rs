use client_server_protocol::Team;
use glam::Vec4;
use rand::{Rng,SeedableRng};
use rand::rngs::StdRng;

use crate::engine::engine_handle::{Command, CommandType, EngineHandle};
use crate::engine::physics::PhysicsSystem;
use crate::transform::Transform;

use super::device::shotgun::{
    LASER_SHOTS_AMOUNT,
    SHOTS_SPREAD
};
use super::shotgun_laser_shot::ShotgunLaserShot;
use super::{Actor, ActorID};




pub struct ShotgunShotSource
{
    pub transform: Transform,
    pub id: Option<ActorID>,
}


impl ShotgunShotSource
{
    pub fn new(
        real_start_position: Vec4,
        visible_start_position: Vec4,
        mut direction: Vec4,
        rng_seed: u64,
        is_replicated: bool,
        damage_dealer_id: ActorID,
        damage_dealer_team: Team,
        engine_handle: &mut EngineHandle,
        physic_system: &PhysicsSystem,
    ) -> Self
    {
        let mut rng = StdRng::seed_from_u64(rng_seed);

        for _ in 0..LASER_SHOTS_AMOUNT
        {
            let direction_deviation = {
                Vec4::new(
                    (rng.random::<f32>()-0.5)*SHOTS_SPREAD,
                    (rng.random::<f32>()-0.5)*SHOTS_SPREAD,
                    (rng.random::<f32>()-0.5)*SHOTS_SPREAD,
                    0.0//(rng.random::<f32>()-0.5)*SHOTS_SPREAD,
                )
            };

            direction = (direction + direction_deviation).normalize();

            if direction.is_nan() {panic!("catched NAN direction vector during making shotgun shot source")}

            let hit = physic_system.ray_cast(
                real_start_position,
                direction,
                700.0,
                Some(damage_dealer_id),
            );

            let possible_destination = if let Some(hit) = hit
            {
                hit.hit_point
            }
            else
            {
                real_start_position + (direction*700.0)
            };

            let shotgun_laser_shot = ShotgunLaserShot::new(
                real_start_position,
                visible_start_position,
                possible_destination,
                damage_dealer_id,
                damage_dealer_team,
                is_replicated,
            );

            engine_handle.send_command(
                Command {
                    sender: damage_dealer_id,
                    command_type: CommandType::SpawnActor(
                        super::ActorWrapper::ShotgunLaserShot(shotgun_laser_shot)
                    )
                }
            );
        }


        ShotgunShotSource
        {
            transform: Transform::from_position(visible_start_position),
            id: None,
        }
    }
}

impl Actor for ShotgunShotSource
{
    fn get_mut_transform(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_id(&self) -> Option<ActorID> {
        self.id
    }

    fn set_id(&mut self, id: ActorID) {
        self.id = Some(id);
    }

    fn tick(
        &mut self,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        audio_system: &mut crate::engine::audio::AudioSystem,
        ui_system: &mut crate::engine::ui::UISystem,
        time_system: &mut crate::engine::time::TimeSystem,
        effects_system: &mut crate::engine::effects::EffectsSystem,
        delta: f32
    )
    {
        let my_id = self.id.expect("Shotgun shot source havn't ActorID");
        engine_handle.send_command(
            Command {
                sender: my_id,
                command_type: CommandType::RemoveActor(my_id)
            }
        );    
    }
}