use fyrox_core::pool::Handle;
use fyrox_sound::source::SoundSource;
use glam::{Vec3, Vec4};

use crate::{
    actor::{
        device::{
            Device,
            DeviceType,
        },
        holegun_miss::HoleGunMiss,
        holegun_shot::HoleGunShot,
        main_player::{
            player_inner_state::PlayerInnerState,
            PlayerMessage, PlayerScreenEffects, PLAYER_PROJECTION_DISPLAY_TIME
        },
        ActorID,
        ActorWrapper,
        Message,
        MessageType,
        SpecificActorMessage
    },
    engine::{
        audio::{
            AudioSystem,
            Sound
        },
        engine_handle::{
            Command,
            CommandType,
            EngineHandle,
        },
        input::ActionsFrameState,
        physics::PhysicsSystem,
        render::VisualElement,
        ui::{
            UIElement,
            UIElementType,
            UISystem
        },
        world::static_object::{
            SphericalVolumeArea,
            VolumeArea
        }
    },
    transform::Transform
};

use client_server_protocol::{
    NetCommand, NetMessageToPlayer, RemoteMessage, Team
};

pub struct HoleGun {
    shooted_on_this_charge: bool,
    is_charging: bool,
    // color: Vec3,
    volume_area: Vec<VolumeArea>,
    shooted_from_pivot_point_dir: Vec4,
    charging_sound: Option<Handle<SoundSource>>,

    energy: f32,
    current_shot_charging_energy: f32,

    energy_gun_hole_size_mult: f32, 
    energy_gun_add_force_mult: f32, 
    energy_gun_damage_mult: f32, 
    energy_gun_restoring_speed: f32,
}

pub const HOLE_GUN_BLUE_COLOR: Vec3 = Vec3::new(0.05, 0.6, 1.6);
pub const HOLE_GUN_RED_COLOR: Vec3 = Vec3::new(2.1, 0.45, 0.05);
pub const CHARGING_COEF: f32 = 0.7;
pub const MAX_CHARGING_TIME: f32 = 3.4;

pub const MAX_ENERGY: f32 = 60.0;
pub const ENERGY_DECREASING_SPEED: f32 = 20.0;
// pub const self.energy_gun_restoring_speed: f32 = 20.0;
pub const ENERGY_SHOT_COST: f32 = 9.0;
// pub const self.energy_gun_hole_size_mult: f32 = 0.1;

impl HoleGun {
    pub fn new(
        energy_gun_hole_size_mult: f32, 
        energy_gun_add_force_mult: f32, 
        energy_gun_damage_mult: f32, 
        energy_gun_restoring_speed: f32,
        shooted_from_pivot_point_dir: Vec4,
    ) -> Self {
        // let color = match team {
        //     Team::Blue => HOLE_GUN_BLUE_COLOR,
        //     Team::Red => HOLE_GUN_RED_COLOR,
        // };

        HoleGun {
            shooted_on_this_charge: false,
            // color,
            volume_area: Vec::with_capacity(1),
            shooted_from_pivot_point_dir,
            is_charging: false,
            charging_sound: None,

            energy: 100.0,
            current_shot_charging_energy: 0.0,

            energy_gun_hole_size_mult, 
            energy_gun_add_force_mult, 
            energy_gun_damage_mult, 
            energy_gun_restoring_speed,
        }
    }

    fn shoot(
        &mut self,
        player_id: ActorID,
        player: &mut PlayerInnerState,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        engine_handle: &mut EngineHandle,
        screen_effects: &mut PlayerScreenEffects,
        charging_energy: f32,
        color: Vec3,
    ) {
        player.crosshair_target_size += 1.0;

        audio_system.remove_sound(
            self.charging_sound.take().expect("Holegun haven't charging sound on shoot")
        );

        let volume_area = self.volume_area.pop().expect("Hole Gun doesn't have volume area on shoot");
        
        audio_system.spawn_non_spatial_sound(
            Sound::HolegunShot,
            (charging_energy*0.9/ (MAX_CHARGING_TIME*2.0)).powf(1.6).clamp(0.1, 0.44), 
        ((MAX_CHARGING_TIME*0.13+1.0) - charging_energy*0.5*0.2) as f64,
            false,
            true,
            fyrox_sound::source::Status::Playing,
        );

        let from = player.get_eyes_position();
                
        let direction = player.transform.get_rotation().inverse() * Vec4::NEG_Z;
    
        let weapon_offset = {
            player.get_eyes_offset() +
            (player.transform.get_rotation().inverse() *
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
            let radius = charging_energy*CHARGING_COEF;

            let hited_players = physic_system.sphere_cast_on_dynamic_colliders(
                position,
                radius,
                None,
            );

            for hit in hited_players {

                if let Some(hited_id) = hit.hited_actors_id
                {
                    if let Some(hited_team) = hit.hited_actors_team
                    {
                        if hited_team != player.team
                        {                            
                            let dist_to_hited_point = {
                                hit.hit_point.distance(position)
                            };
            
                            let damage = ((radius * 54.0) / (1.0 + dist_to_hited_point*0.3)) * 
                                self.energy_gun_damage_mult;
            
                            let force = (hit.hit_normal * damage / -4.5) *
                                self.energy_gun_add_force_mult;
            
                            engine_handle.send_direct_message(
                                hited_id,
                                Message {
                                    from: player_id,
                                    remote_sender: false,
                                    message: MessageType::SpecificActorMessage(
                                        SpecificActorMessage::PlayerMessage(
                                            PlayerMessage::DealDamageAndAddForce(
                                                damage as u32,
                                                force,
                                                hit.hit_point,
                                                player.team
                                            )
                                        )
                                    )
                                }
                            );
                        }
                    }
                }
            }

            let base_coef = 
            {
                let mut coef = f32::clamp(
                    (position.w - player.blue_map_w_level) /
                    (player.red_map_w_level - player.blue_map_w_level),
                     0.0,
                     1.0
                );

                if player.team == Team::Blue
                {
                    coef = 1.0 - coef;
                }

                coef = (coef * 0.38) + 0.62;

                coef
            };

            let hole = HoleGunShot::new(
                position,
                shooted_from,
                radius.abs()*base_coef,
                color,
                volume_area,
                1.0,
            );
    
            engine_handle.send_command(
                Command {
                    sender: player_id,
                    command_type: CommandType::SpawnActor(
                        ActorWrapper::HoleGunShot(hole)
                    )
                }
            );

            engine_handle.send_command(
                Command {
                    sender: player_id,
                    command_type: CommandType::NetCommand(
                        NetCommand::SendBoardcastNetMessageReliable(
                            NetMessageToPlayer::RemoteDirectMessage(
                                player_id,
                                RemoteMessage::SpawnHoleGunShotActor(
                                    position.to_array(),
                                    shooted_from.to_array(),
                                    radius.abs()*base_coef,
                                    color.to_array(),
                                    volume_area_radius.abs(),
                                )
                            )
                        )
                    )
                }
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

            engine_handle.send_command(
                Command {
                    sender: player_id,
                    command_type: CommandType::SpawnActor(
                        ActorWrapper::HoleGunMiss(miss)
                    )
                }
            );

            engine_handle.send_command(
                Command {
                    sender: player_id,
                    command_type: CommandType::NetCommand(
                        NetCommand::SendBoardcastNetMessageReliable(
                            NetMessageToPlayer::RemoteDirectMessage(
                                player_id,
                                RemoteMessage::SpawHoleGunMissActor(
                                    position_remote.to_array(),
                                    shooted_from.to_array(),
                                    radius.abs(),
                                    color.to_array(),
                                    volume_area_radius.abs(),
                                )
                            )
                        )
                    )
                }
            )
        }
    }
}


impl Device for HoleGun {
    fn get_device_type(&self) -> DeviceType {
        DeviceType::Gun
    }

    fn get_visual_element<'a>(&'a self, transform: &'a Transform) -> Option<VisualElement<'a>> {
        Some(
            VisualElement {
                transform,
                static_objects: None,
                coloring_areas: None,
                volume_areas: Some(&self.volume_area),
                waves: None,
                player: None,
            }
        )
    }

    fn process_while_player_is_not_alive(
            &mut self,
            player_id: ActorID,
            player: &mut PlayerInnerState,
            input: &ActionsFrameState,
            physic_system: &PhysicsSystem,
            audio_system: &mut AudioSystem,
            ui_system: &mut UISystem,
            engine_handle: &mut EngineHandle,
            delta: f32,
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
        engine_handle: &mut EngineHandle,
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
                        crate::engine::audio::Sound::HolegunCharging,
                        0.38,
                        1.2,
                        false,
                        true,
                        fyrox_sound::source::Status::Playing
                    ));

    
                    let shooted_from_offset = {
                        (player.get_eyes_offset()) +
                        (player.transform.get_rotation().inverse() *
                        (self.shooted_from_pivot_point_dir.normalize() * player.collider.get_collider_radius()))
                    };
    
                    let volume_area = VolumeArea::SphericalVolumeArea(
                        SphericalVolumeArea {
                            color: color,
                            translation: shooted_from_offset,
                            radius: 0.05,
                        }
                    );
    
                    self.volume_area.push(volume_area);

                    engine_handle.send_command(
                        Command {
                            sender: player_id,
                            command_type: CommandType::NetCommand(
                                NetCommand::SendBoardcastNetMessageReliable(
                                    NetMessageToPlayer::RemoteDirectMessage(
                                        player_id,
                                        RemoteMessage::HoleGunStartCharging
                                    )
                                )
                            )
                        }
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
                
                match &mut self.volume_area[0] {
                    
                    VolumeArea::SphericalVolumeArea(area) => {
                        let shooted_from_offset = {
                            (player.get_eyes_offset()) +
                            (player.transform.get_rotation().inverse() *
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
                        engine_handle,
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
                        engine_handle,
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
            
                    let volume_area = self.volume_area.pop().expect("Hole Gun doesn't have volume area on shoot");

                    self.current_shot_charging_energy = 0.0;

                }

                self.is_charging = false;
            }

            self.energy += delta*self.energy_gun_restoring_speed;

            self.energy = self.energy.clamp(0.0, MAX_ENERGY);
        }

        let bar = match player.team {
            Team::Red => ui_system.get_mut_ui_element(&UIElementType::EnergyGunBarRed),
            Team::Blue => ui_system.get_mut_ui_element(&UIElementType::EnergyGunBarBlue),
        };

        if let UIElement::ProgressBar(bar) = bar {
            bar.set_bar_value(self.energy / MAX_ENERGY)
        }
    }

    fn process_while_deactive(
            &mut self,
            player_id: ActorID,
            player: &mut PlayerInnerState,
            input: &ActionsFrameState,
            physic_system: &PhysicsSystem,
            audio_system: &mut AudioSystem,
            ui_system: &mut UISystem,
            engine_handle: &mut EngineHandle,
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
        engine_handle: &mut EngineHandle,
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
                    engine_handle,
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

        self.volume_area.clear();


        let bar = ui_system.get_mut_ui_element(&UIElementType::EnergyGunBarRed);

        if let UIElement::ProgressBar(bar) = bar {
            *bar.ui_data.is_visible.lock().unwrap() = false;
        }

        let bar = ui_system.get_mut_ui_element(&UIElementType::EnergyGunBarBlue);

        if let UIElement::ProgressBar(bar) = bar {
            *bar.ui_data.is_visible.lock().unwrap() = false;
        }

        let img = ui_system.get_mut_ui_element(&UIElementType::EnergyGunImage);

        if let UIElement::Image(img) = img {
            *img.ui_data.is_visible.lock().unwrap() = false;
        }
    }

    fn activate(
        &mut self,
        player_id: ActorID,
        player: &mut PlayerInnerState,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        engine_handle: &mut EngineHandle,
    ) {
        let img = ui_system.get_mut_ui_element(&UIElementType::EnergyGunImage);

        if let UIElement::Image(img) = img {
            *img.ui_data.is_visible.lock().unwrap() = true;
        }

        match player.team
        {
            Team::Red =>
            {
                let bar = ui_system.get_mut_ui_element(&UIElementType::EnergyGunBarRed);

                if let UIElement::ProgressBar(bar) = bar {
                    *bar.ui_data.is_visible.lock().unwrap() = true;
                }

                let bar = ui_system.get_mut_ui_element(&UIElementType::EnergyGunBarBlue);

                if let UIElement::ProgressBar(bar) = bar {
                    *bar.ui_data.is_visible.lock().unwrap() = false;
                }
            }

            Team::Blue =>
            {
                let bar = ui_system.get_mut_ui_element(&UIElementType::EnergyGunBarBlue);

                if let UIElement::ProgressBar(bar) = bar {
                    *bar.ui_data.is_visible.lock().unwrap() = true;
                }

                let bar = ui_system.get_mut_ui_element(&UIElementType::EnergyGunBarRed);

                if let UIElement::ProgressBar(bar) = bar {
                    *bar.ui_data.is_visible.lock().unwrap() = false;
                }
            }
        }
    }
}