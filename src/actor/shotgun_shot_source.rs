use client_server_protocol::Team;
use glam::Vec4;
use fyrox_core::rand::{Rng, RngCore, SeedableRng};
use fyrox_core::rand::prelude::StdRng;

use crate::engine::audio::AudioSystem;
use crate::engine::engine_handle::{Command, CommandType, EngineHandle};
use crate::engine::physics::PhysicsSystem;
use crate::engine::render::VisualElement;
use crate::engine::world::static_object::{SphericalVolumeArea, VolumeArea};
use crate::transform::Transform;

use super::device::shotgun::{
    SHOTGUN_LASER_SHOTS_AMOUNT, SHOTGUN_LASER_SHOTS_AMOUNT_WITHOUT_W_SPREAD, SHOTGUN_LASER_SHOT_COLOR, SHOTGUN_LASER_SHOT_MAX_DISTANCE, SHOTGUN_SHOTS_SPREAD, SHOTGUN_SHOTS_SPREAD_ALONG_W, SHOTGUN_SHOT_FLASH_EXPLAND_SPEED, SHOTGUN_SHOT_FLASH_FADE_SPEED, SHOTGUN_SHOT_FLASH_MAX_RADIUS
};
use super::shotgun_laser_shot::ShotgunLaserShot;
use super::{Actor, ActorID};




pub struct ShotgunShotSource
{
    transform: Transform,
    id: Option<ActorID>,
    volume_areas: Vec<VolumeArea>,
    flash_max_size_reached: bool,
    beam_and_flash_size_mult: f32,
}


impl ShotgunShotSource
{
    pub fn new(
        real_start_position: Vec4,
        visible_start_position: Vec4,
        direction: Vec4,
        rng_seed: u64,
        is_replicated: bool,
        damage_dealer_id: ActorID,
        damage_dealer_team: Team,
        beam_and_flash_size_mult: f32,
        it_is_for_2d_3d_example: bool,
        engine_handle: &mut EngineHandle,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
    ) -> Self
    {
        if is_replicated
        {
            audio_system.spawn_spatial_sound(
                crate::engine::audio::Sound::ShotgunShot,
                0.25,
                1.0,
                false,
                true,
                fyrox_sound::source::Status::Playing,
                visible_start_position,
                1.0,
                1.0,
                50.0
            );
        }
        else
        {
            audio_system.spawn_non_spatial_sound(
                crate::engine::audio::Sound::ShotgunShot,
                0.36,
                1.0,
                false,
                true,
                fyrox_sound::source::Status::Playing,
            );
        }


        let mut rng = StdRng::seed_from_u64(rng_seed);

        let shots_amount = if it_is_for_2d_3d_example
        {
            SHOTGUN_LASER_SHOTS_AMOUNT / 2
        }
        else
        {
            SHOTGUN_LASER_SHOTS_AMOUNT
        };

        let shots_amount_without_spread = if it_is_for_2d_3d_example
        {
            SHOTGUN_LASER_SHOTS_AMOUNT_WITHOUT_W_SPREAD / 2
        }
        else
        {
            SHOTGUN_LASER_SHOTS_AMOUNT_WITHOUT_W_SPREAD
        };

        for i in 0..shots_amount
        {
            let mut direction = direction;

            let direction_deviation = if it_is_for_2d_3d_example
            {
                if i <= shots_amount_without_spread
                {
                    Vec4::new(
                        0.0,
                        rng.gen_range(-SHOTGUN_SHOTS_SPREAD..=SHOTGUN_SHOTS_SPREAD),
                        rng.gen_range(-SHOTGUN_SHOTS_SPREAD..=SHOTGUN_SHOTS_SPREAD),
                        0.0,
                    )
                }
                else
                {
                    Vec4::new(
                        rng.gen_range(-SHOTGUN_SHOTS_SPREAD_ALONG_W..=SHOTGUN_SHOTS_SPREAD_ALONG_W),
                        rng.gen_range(-SHOTGUN_SHOTS_SPREAD..=SHOTGUN_SHOTS_SPREAD),
                        rng.gen_range(-SHOTGUN_SHOTS_SPREAD..=SHOTGUN_SHOTS_SPREAD),
                        0.0,
                    )
                }
            }
            else
            {
                if i <= SHOTGUN_LASER_SHOTS_AMOUNT_WITHOUT_W_SPREAD
                {
                    Vec4::new(
                        rng.gen_range(-SHOTGUN_SHOTS_SPREAD..=SHOTGUN_SHOTS_SPREAD),
                        rng.gen_range(-SHOTGUN_SHOTS_SPREAD..=SHOTGUN_SHOTS_SPREAD),
                        rng.gen_range(-SHOTGUN_SHOTS_SPREAD..=SHOTGUN_SHOTS_SPREAD),
                        0.0,
                    )
                }
                else
                {
                    Vec4::new(
                        rng.gen_range(-SHOTGUN_SHOTS_SPREAD..=SHOTGUN_SHOTS_SPREAD),
                        rng.gen_range(-SHOTGUN_SHOTS_SPREAD..=SHOTGUN_SHOTS_SPREAD),
                        rng.gen_range(-SHOTGUN_SHOTS_SPREAD..=SHOTGUN_SHOTS_SPREAD),
                        rng.gen_range(-SHOTGUN_SHOTS_SPREAD_ALONG_W..=SHOTGUN_SHOTS_SPREAD_ALONG_W),
                    )
                }
            };

            direction = (direction + direction_deviation).normalize();

            if direction.is_nan() {panic!("catched NAN direction vector during making shotgun shot source")}

            let hit = physic_system.ray_cast(
                real_start_position,
                direction,
                SHOTGUN_LASER_SHOT_MAX_DISTANCE,
                Some(damage_dealer_id),
            );

            let possible_destination = if let Some(hit) = hit
            {
                hit.hit_point
            }
            else
            {
                real_start_position + (direction*SHOTGUN_LASER_SHOT_MAX_DISTANCE)
            };

            let shotgun_laser_shot = ShotgunLaserShot::new(
                real_start_position,
                visible_start_position,
                possible_destination,
                damage_dealer_id,
                damage_dealer_team,
                is_replicated,
                beam_and_flash_size_mult
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

        let flash = VolumeArea::SphericalVolumeArea(
            SphericalVolumeArea {
                translation: Vec4::ZERO,
                radius: 0.01,
                color: SHOTGUN_LASER_SHOT_COLOR,
            }
        );

        let mut volume_areas = Vec::with_capacity(1);
        volume_areas.push(flash);

        ShotgunShotSource
        {
            transform: Transform::from_position(visible_start_position),
            id: None,
            volume_areas,
            flash_max_size_reached: false,
            beam_and_flash_size_mult,
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

    fn get_visual_element(&self) -> Option<VisualElement> {
        Some(VisualElement {
            transform: &self.transform,
            static_objects: None,
            coloring_areas: None,
            volume_areas: Some(&self.volume_areas),
            waves: None,
            player: None,
            child_visual_elem: None,
        })
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
        if self.flash_max_size_reached
        {
            if let VolumeArea::SphericalVolumeArea(flash) =
                &mut self.volume_areas[0]
            {
                flash.radius -= delta*SHOTGUN_SHOT_FLASH_FADE_SPEED*self.beam_and_flash_size_mult;

                if flash.radius <= 0.01
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
        }
        else
        {
            if let VolumeArea::SphericalVolumeArea(flash) =
                &mut self.volume_areas[0]
            {
                flash.radius += delta*SHOTGUN_SHOT_FLASH_EXPLAND_SPEED*self.beam_and_flash_size_mult;

                if flash.radius >= SHOTGUN_SHOT_FLASH_MAX_RADIUS
                {
                    self.flash_max_size_reached = true;
                }
            }
        }  
    }
}