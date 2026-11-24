// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

// The obstacles gun isn't given to players in the current game modes; kept for later.
#![allow(dead_code)]





use crate::engine::shared_systems::audio::AudioSystem;
use crate::engine::shared_systems::resources::{ResourceLoader, Retention};
use fyrox_sound::buffer::SoundBufferResource;
use crate::actor::session_controller::send_to_session_controller;
use crate::actor::session_controller::SessionControllerMessage;
use crate::engine::shared_systems::net::Reliability;

use fyrox_core::pool::Handle;
use fyrox_sound::source::SoundSource;
use glam::{Vec3, Vec4};

use crate::{
    actor::{
        ActorID, ActorWrapper, device::{
            Device,
            DeviceType,
        }, holegun_miss::HoleGunMiss, main_player::{
            PlayerScreenEffects, player_inner_state::PlayerInnerState
        }, obstaclesgun_shot::ObstaclesGunShot
    },
    engine::{
        world::messenger::{
            Command,
            CommandType,
            Messenger,
        },
        shared_systems::input::ActionsFrameState,
        world::physics::PhysicsSystem,
        shared_systems::render::ChildVisualElement,
        shared_systems::ui::{
            UIElement,
            UIElementType,
            UISystem
        },
        world::static_object::{
            SphericalVolumeArea,
            VolumeArea
        }
    },
    engine::transform::{FORWARD, Transform}
};

use net_protocol::gameplay::{
    NetMessageToPlayer, RemoteMessage, Team
};

pub struct ObstaclesGun {
    holegun_charging_sound: Option<SoundBufferResource>,
    obstacle_gun_shot_sound: Option<SoundBufferResource>,

    shooted_on_this_charge: bool,
    is_charging: bool,
    // color: Vec3,
    shooted_from_pivot_point_dir: Vec4,
    charging_sound: Option<Option<Handle<SoundSource>>>,

    energy: f32,
    current_shot_charging_energy: f32,

    energy_gun_hole_size_mult: f32, 
    energy_gun_add_force_mult: f32, 
    energy_gun_damage_mult: f32, 
    energy_gun_restoring_speed: f32,

    visual_element: ChildVisualElement,
}

pub const HOLE_GUN_BLUE_COLOR: Vec3 = Vec3::new(0.05, 0.6, 1.6);
pub const HOLE_GUN_RED_COLOR: Vec3 = Vec3::new(2.1, 0.45, 0.05);
pub const CHARGING_COEF: f32 = 0.7;
pub const MAX_CHARGING_TIME: f32 = 3.4;

pub const MAX_ENERGY: f32 = 60.0;
pub const ENERGY_DECREASING_SPEED: f32 = 20.0;
pub const ENERGY_SHOT_COST: f32 = 9.0;

impl ObstaclesGun {
    pub fn new(
        resources: &mut ResourceLoader,
        energy_gun_hole_size_mult: f32, 
        energy_gun_add_force_mult: f32, 
        energy_gun_damage_mult: f32, 
        mut energy_gun_restoring_speed: f32,
        shooted_from_pivot_point_dir: Vec4,
    ) -> Self {

        let visual_element = ChildVisualElement
        {
            static_objects: None,
            coloring_areas: None,
            volume_areas: Some(Vec::with_capacity(1)),
            waves: None,
            player: None,
        };

        energy_gun_restoring_speed *= 0.65;

        ObstaclesGun {
            holegun_charging_sound: resources.load_media(
                crate::embedded_asset!("sounds/holegun_charging.wav"), Retention::Session,
            ),
            obstacle_gun_shot_sound: resources.load_media(
                crate::embedded_asset!("sounds/obstacle_gun_shot.wav"), Retention::Session,
            ),

            shooted_on_this_charge: false,
            // color,
            shooted_from_pivot_point_dir,
            is_charging: false,
            charging_sound: None,

            energy: 100.0,
            current_shot_charging_energy: 0.0,

            energy_gun_hole_size_mult, 
            energy_gun_add_force_mult, 
            energy_gun_damage_mult, 
            energy_gun_restoring_speed,

            visual_element,
        }
    }

    fn shoot(
        &mut self,
        player_id: ActorID,
        player: &mut PlayerInnerState,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        messenger: &mut Messenger,
        _screen_effects: &mut PlayerScreenEffects,
        charging_energy: f32,
        color: Vec3,
    ) {
        player.crosshair_target_size += 1.0;

        audio_system.remove_sound(
            self.charging_sound.take().expect("Holegun haven't charging sound on shoot")
        );

        let volume_area = self
            .visual_element
            .volume_areas
            .as_mut()
            .expect("Hole gun have not bolume areas in visual element")
            .pop().expect("Hole Gun doesn't have volume area on shoot");

        audio_system.spawn_non_spatial_sound(
            &self.obstacle_gun_shot_sound,
            (charging_energy*2.9/ (MAX_CHARGING_TIME*1.0)).powf(0.7).clamp(0.2, 1.94), 
            ((MAX_CHARGING_TIME*0.1+1.15) - charging_energy*0.5*0.15).clamp(1.0, 100.0) as f64,
            false,
            true,
            fyrox_sound::source::Status::Playing,
        );

        let from = player.get_eyes_position();
                
        let direction = player.transform.get_rotation() * FORWARD;
    
        let weapon_offset = {
            player.get_eyes_offset() +
            (player.transform.get_rotation() *
            (self.shooted_from_pivot_point_dir.normalize() * player.collider.get_collider_radius()))
        };
        
        let volume_area_radius = match &volume_area {
            VolumeArea::SphericalVolumeArea(area) => {
                area.radius
            },
            _ => {panic!("Charging volume area in holegun is not SphericalVolumeArea")}
        };

        let hit = physic_system.ray_cast(from, direction, 700.0, Some(player_id));

        if let Some(hit) = hit {

            let position = hit.hit_point;
            let shooted_from = player.transform.get_position() + weapon_offset;
            let radius = charging_energy*CHARGING_COEF*0.79;

            let obstacle = ObstaclesGunShot::new(
                position,
                shooted_from,
                radius.abs(),
                color,
                volume_area,
                1.0,
            );
    
            messenger.send_command(
                Command {
                    sender: player_id,
                    command_type: CommandType::SpawnActor(
                        Box::new(ActorWrapper::ObstaclesGunShot(obstacle))
                    )
                }
            );

            send_to_session_controller(
                messenger,
                player_id,
                SessionControllerMessage::SendToPlayers(
                    NetMessageToPlayer::RemoteDirectMessage(
                        player_id,
                        RemoteMessage::SpawnObstacleGunShotActor(
                            position.to_array(),
                            shooted_from.to_array(),
                            radius.abs(),//*base_coef,
                            color.to_array(),
                            volume_area_radius.abs(),
                        )
                    ),
                    Reliability::Reliable,
                ),
            )

        } else {
            let position_local = from + (direction * 17.0);
            let position_remote = from + (direction * 1500.0);
            let shooted_from = player.transform.get_position() + weapon_offset;
            let radius = charging_energy*CHARGING_COEF;

            let miss = HoleGunMiss::new(
                position_local,
                shooted_from,
                radius.abs(),
                color,
                volume_area,
                1.0,
            );

            messenger.send_command(
                Command {
                    sender: player_id,
                    command_type: CommandType::SpawnActor(
                        Box::new(ActorWrapper::HoleGunMiss(miss))
                    )
                }
            );

            send_to_session_controller(
                messenger,
                player_id,
                SessionControllerMessage::SendToPlayers(
                    NetMessageToPlayer::RemoteDirectMessage(
                        player_id,
                        RemoteMessage::SpawHoleGunMissActor(
                            position_remote.to_array(),
                            shooted_from.to_array(),
                            radius.abs(),
                            color.to_array(),
                            volume_area_radius.abs(),
                        )
                    ),
                    Reliability::Reliable,
                ),
            )
        }
    }
}


impl Device for ObstaclesGun {
    fn get_device_type(&self) -> DeviceType {
        DeviceType::Gun
    }

    fn get_visual_element<'a>(&'a self, _transform: &'a Transform) -> Option<&'a ChildVisualElement> {
        Some(
            &self.visual_element
        )
    }

    fn process_while_player_is_not_alive(
            &mut self,
            _player_id: ActorID,
            _player: &mut PlayerInnerState,
            _input: &ActionsFrameState,
            _physic_system: &PhysicsSystem,
            _audio_system: &mut AudioSystem,
            _ui_system: &mut UISystem,
            _messenger: &mut Messenger,
            _delta: f32,
        ) {
        
    }

    fn process_input(
        &mut self,
        player_id: ActorID,
        player: &mut PlayerInnerState,
        screen_effects: &mut PlayerScreenEffects,
        input: &ActionsFrameState,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        messenger: &mut Messenger,
        delta: f32,
    )
    {
        let color = match player.team {
            Team::Red => HOLE_GUN_RED_COLOR,
            Team::Blue => HOLE_GUN_BLUE_COLOR,
        };

        if input.first_mouse.is_action_pressed() {

            if self.energy < ENERGY_SHOT_COST {

                self.energy += delta*self.energy_gun_restoring_speed;
                self.energy = self.energy.clamp(0.0, MAX_ENERGY);

            } else {

                if !self.is_charging {
                    self.is_charging = true;
                    
                    // start charging

                    self.charging_sound = Some(audio_system.spawn_non_spatial_sound(
                        &self.holegun_charging_sound,
                        0.38,
                        1.2,
                        false,
                        true,
                        fyrox_sound::source::Status::Playing
                    ));

    
                    let shooted_from_offset = {
                        (player.get_eyes_offset()) +
                        (player.transform.get_rotation() *
                        (self.shooted_from_pivot_point_dir.normalize() * player.collider.get_collider_radius()))
                    };
    
                    let volume_area = VolumeArea::SphericalVolumeArea(
                        SphericalVolumeArea {
                            color: color,
                            translation: shooted_from_offset,
                            radius: 0.05,
                        }
                    );
    
                    self
                        .visual_element
                        .volume_areas
                        .as_mut()
                        .expect("Hole gun have not bolume areas in visual element")
                        .push(volume_area);

                    send_to_session_controller(
                        messenger,
                        player_id,
                        SessionControllerMessage::SendToPlayers(
                            NetMessageToPlayer::RemoteDirectMessage(
                                player_id,
                                RemoteMessage::HoleGunStartCharging
                            ),
                            Reliability::Reliable,
                        ),
                    )
                }

                self.energy -= delta * ENERGY_DECREASING_SPEED;

                self.energy = self.energy.clamp(0.0, MAX_ENERGY);
                
                self.current_shot_charging_energy += (delta*ENERGY_DECREASING_SPEED).min(self.energy);


                // audio_system.sound_set_pitch_and_gain(
                //     self.charging_sound.expect("Holegun have not charging sound on charging"),
                //     1.3 + (self.charging_time*0.5) as f64,
                //     0.4 + (self.charging_time*0.13),
                // );
                
                match &mut self
                    .visual_element
                    .volume_areas
                    .as_mut()
                    .expect("Hole gun have not bolume areas in visual element")
                    [0]
                {
                    
                    VolumeArea::SphericalVolumeArea(area) => {
                        let shooted_from_offset = {
                            (player.get_eyes_offset()) +
                            (player.transform.get_rotation() *
                            (self.shooted_from_pivot_point_dir.normalize() * player.collider.get_collider_radius()))
                        };

                        area.radius = self.current_shot_charging_energy * 0.003;
                        area.translation = shooted_from_offset;
                    }
                    _ => {
                        panic!("charging volume area in HoleGun is not SphericalVolumeArea")
                    }
                }
    
                if self.energy < ENERGY_SHOT_COST+ENERGY_DECREASING_SPEED*delta &&
                    self.energy > ENERGY_SHOT_COST-ENERGY_DECREASING_SPEED*delta
                {
    
                    self.shooted_on_this_charge = true;
                    
                    self.shoot(
                        player_id,
                        player,
                        physic_system,
                        audio_system,
                        messenger,
                        screen_effects,
                        self.current_shot_charging_energy*self.energy_gun_hole_size_mult+ENERGY_SHOT_COST*0.04,
                        color,
                    );
    
                    self.current_shot_charging_energy = 0.0;
                    self.energy = 0.0;
                    
                    self.is_charging = false;

                }


            }

        } else {

            self.shooted_on_this_charge = false;

            if self.is_charging {

                if self.current_shot_charging_energy > 0.0 && self.energy >= ENERGY_SHOT_COST
                {
                    self.shoot(
                        player_id,
                        player,
                        physic_system,
                        audio_system,
                        messenger,
                        screen_effects,
                        self.current_shot_charging_energy*self.energy_gun_hole_size_mult+ENERGY_SHOT_COST*0.04,
                        color,
                    );
    
                    self.current_shot_charging_energy = 0.0;
                    self.energy -= ENERGY_SHOT_COST;
                    self.energy = self.energy.clamp(0.0, MAX_ENERGY);
                    
                } else {

                    audio_system.remove_sound(
                        self.charging_sound.take().expect("Holegun haven't charging sound on shoot")
                    );
            
                    let _volume_area = self
                        .visual_element
                        .volume_areas
                        .as_mut()
                        .expect("Hole gun have not bolume areas in visual element")
                        .pop()
                        .expect("Hole Gun doesn't have volume area on shoot");

                    self.current_shot_charging_energy = 0.0;

                }

                self.is_charging = false;
            }

            self.energy += delta*self.energy_gun_restoring_speed;

            self.energy = self.energy.clamp(0.0, MAX_ENERGY);
        }

        let bar = match player.team {
            Team::Red => ui_system.get_mut_ui_element(&UIElementType::ObstacleGunBarRed),
            Team::Blue => ui_system.get_mut_ui_element(&UIElementType::ObstacleGunBarBlue),
        };

        if let UIElement::ProgressBar(bar) = bar {
            bar.set_bar_value(self.energy / MAX_ENERGY)
        }
    }

    fn process_while_deactive(
            &mut self,
            _player_id: ActorID,
            _player: &mut PlayerInnerState,
            _input: &ActionsFrameState,
            _physic_system: &PhysicsSystem,
            _audio_system: &mut AudioSystem,
            _ui_system: &mut UISystem,
            _messenger: &mut Messenger,
            delta: f32,
        ) {
            self.energy += delta*self.energy_gun_restoring_speed;

            self.energy = self.energy.clamp(0.0, MAX_ENERGY);
    }


    fn deactivate(
        &mut self,
        player_id: ActorID,
        player: &mut PlayerInnerState,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        messenger: &mut Messenger,
        screen_effects: &mut PlayerScreenEffects,
    ) {
        let color = match player.team {
            Team::Red => HOLE_GUN_RED_COLOR,
            Team::Blue => HOLE_GUN_BLUE_COLOR,
        };

        self.shooted_on_this_charge = false;
        
        if self.is_charging {
            if self.current_shot_charging_energy > 0.0 {

                self.shoot(
                    player_id,
                    player,
                    physic_system,
                    audio_system,
                    messenger,
                    screen_effects,
                    self.current_shot_charging_energy*self.energy_gun_hole_size_mult+ENERGY_SHOT_COST*0.04,
                    color,
                );

                self.current_shot_charging_energy = 0.0;
                self.energy -= ENERGY_SHOT_COST;
                self.energy = self.energy.clamp(0.0, MAX_ENERGY);
            }
            self.is_charging = false;
        }

        self
            .visual_element
            .volume_areas
            .as_mut()
            .expect("Hole gun have not bolume areas in visual element")
            .clear();


        let bar = ui_system.get_mut_ui_element(&UIElementType::ObstacleGunBarRed);

        if let UIElement::ProgressBar(bar) = bar {
            bar.ui_data.is_visible = false;
        }

        let bar = ui_system.get_mut_ui_element(&UIElementType::ObstacleGunBarBlue);

        if let UIElement::ProgressBar(bar) = bar {
            bar.ui_data.is_visible = false;
        }

        let img = ui_system.get_mut_ui_element(&UIElementType::ObstacleGunImage);

        if let UIElement::Image(img) = img {
            img.ui_data.is_visible = false;
        }
    }

    fn activate(
        &mut self,
        _player_id: ActorID,
        player: &mut PlayerInnerState,
        _physic_system: &PhysicsSystem,
        _audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        _messenger: &mut Messenger,
    ) {
        let img = ui_system.get_mut_ui_element(&UIElementType::ObstacleGunImage);

        if let UIElement::Image(img) = img {
            img.ui_data.is_visible = true;
        }

        match player.team
        {
            Team::Red =>
            {
                let bar = ui_system.get_mut_ui_element(&UIElementType::ObstacleGunBarRed);

                if let UIElement::ProgressBar(bar) = bar {
                    bar.ui_data.is_visible = true;
                }

                let bar = ui_system.get_mut_ui_element(&UIElementType::ObstacleGunBarBlue);

                if let UIElement::ProgressBar(bar) = bar {
                    bar.ui_data.is_visible = false;
                }
            }

            Team::Blue =>
            {
                let bar = ui_system.get_mut_ui_element(&UIElementType::ObstacleGunBarBlue);

                if let UIElement::ProgressBar(bar) = bar {
                    bar.ui_data.is_visible = true;
                }

                let bar = ui_system.get_mut_ui_element(&UIElementType::ObstacleGunBarRed);

                if let UIElement::ProgressBar(bar) = bar {
                    bar.ui_data.is_visible = false;
                }
            }
        }
    }
}