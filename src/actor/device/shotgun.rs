use glam::{Vec3, Vec4};

use crate::{
    actor::{
        main_player::{
            player_inner_state::PlayerInnerState,
            PlayerScreenEffects,
        }, shotgun_shot_source::ShotgunShotSource, ActorID, ActorWrapper
    },
    engine::{
        audio::AudioSystem,
        engine_handle::{
            Command,
            CommandType,
            EngineHandle
        },
        input::ActionsFrameState,
        physics::PhysicsSystem,
        render::ChildVisualElement,
        ui::{
            UIElement,
            UIElementType,
            UISystem
        }
    },
    transform::{Transform, FORWARD}
};

use client_server_protocol::{
    NetCommand, NetMessageToPlayer, RemoteMessage, Team
};

use super::{Device, DeviceType};


const MAX_TEMPERTURE: f32 = 39.1;
const MAX_TEMPERTURE_FOR_SHOOT: f32 = 0.0;
const ADD_TEMPERTURE_ON_SHHOT: f32 = 39.1;
const CROSSHAIR_INCREASE_ON_SHOOT: f32 = 1.0;

pub const SHOTGUN_LASER_SHOT_HOLE_REDUCTION_SPEED: f32 = 0.35;
pub const SHOTGUN_LASER_SHOT_EXPLOSION_EXPAND_SPEED: f32 = 6.2;
pub const SHOTGUN_LASER_SHOT_EXPLOSION_MAX_RADIUS: f32 = 0.25;
pub const SHOTGUN_LASER_SHOT_EXPLOSION_HOLE_MULT: f32 = 0.4;
pub const SHOTGUN_LASER_SHOT_MAX_DISTANCE: f32 = 200.0;
pub const SHOTGUN_LASER_SHOT_DAMAGE: u32 = 8;
pub const SHOTGUN_LASER_SHOT_ADD_FORCE_PER_HIT: f32 = 1.0;
pub const SHOTGUN_LASER_SHOT_SPEED: f32 = 155.5;
pub const SHOTGUN_LASER_SHOT_LENGTH: f32 = 7.6;
pub const SHOTGUN_LASER_SHOT_BEAM_RADIUS: f32 = 0.045;
pub const SHOTGUN_LASER_SHOT_COLOR: Vec3 = Vec3::new(1.0, 0.3, 0.0);
pub const SHOTGUN_SHOT_FLASH_EXPLAND_SPEED: f32 = 2.5;
pub const SHOTGUN_SHOT_FLASH_MAX_RADIUS: f32 = 0.12;
pub const SHOTGUN_SHOT_FLASH_FADE_SPEED: f32 = 2.5;

pub const SHOTGUN_LASER_SHOTS_AMOUNT: u32 = 19;
pub const SHOTGUN_LASER_SHOTS_AMOUNT_WITHOUT_W_SPREAD: u32 = 5;
pub const SHOTGUN_SHOTS_SPREAD: f32 = 0.093;
pub const SHOTGUN_SHOTS_SPREAD_ALONG_W: f32 = 0.175;

const SHOTGUN_COOLING_SPEED: f32 = 40.0;

pub struct Shotgun {
    temperature: f32,
    shooted_from_pivot_point_dir: Vec4,
    this_is_shotgun_for_2d_3d_example: bool,
}

impl Shotgun {
    pub fn new(
        shooted_from_pivot_point_dir: Vec4,
        this_is_shotgun_for_2d_3d_example: bool,

    ) -> Self {

        Shotgun {
            temperature: 0.0,
            shooted_from_pivot_point_dir,
            this_is_shotgun_for_2d_3d_example
        }
    }

    fn shoot(
        &mut self,
        player_id: ActorID,
        player: &mut PlayerInnerState,
        screen_effects: &mut PlayerScreenEffects,
        physic_system: &PhysicsSystem,
        audio_system: &mut AudioSystem,
        engine_handle: &mut EngineHandle,
    )
    {
        self.temperature += ADD_TEMPERTURE_ON_SHHOT;

        player.crosshair_target_size += CROSSHAIR_INCREASE_ON_SHOOT;

        let rng_seed: u64 = fyrox_core::rand::random();

        let weapon_offset = {
            player.get_eyes_offset() +
            (player.transform.get_rotation() *
            (self.shooted_from_pivot_point_dir.normalize() * player.collider.get_collider_radius()))
        };

        let real_start_position = player.get_eyes_position();
        let visible_start_position = player.transform.get_position() + weapon_offset;
        let direction = player.transform.get_rotation() * FORWARD;

        let shotgun_shot_source = ShotgunShotSource::new(
            real_start_position,
            visible_start_position,
            direction,
            rng_seed,
            false,
            player_id,
            player.team,
            1.0,
            self.this_is_shotgun_for_2d_3d_example,
            engine_handle,
            physic_system,
            audio_system,
        );


        engine_handle.send_command(
            Command {
                sender: player_id,
                command_type: CommandType::SpawnActor(
                    ActorWrapper::ShotgunShotSource(shotgun_shot_source)
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
                            RemoteMessage::SpawnShotgunShot(
                                real_start_position.to_array(),
                                direction.to_array(),
                                rng_seed,
                                player_id,
                                player.team,
                            )
                        )
                    )
                )
            }
        )
    }

    fn cool_shotgun(&mut self, delta: f32) {
        if self.temperature > delta * SHOTGUN_COOLING_SPEED {
            self.temperature -= delta * SHOTGUN_COOLING_SPEED;
        } else {
            self.temperature = 0.0;
        }
    }
}

impl Device for Shotgun {
    fn get_device_type(&self) -> DeviceType {
        DeviceType::Gun
    }

    fn get_visual_element<'a>(&'a self, transform: &'a Transform) -> Option<&'a ChildVisualElement> {
        None
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
    ) {
        if input.first_mouse.is_action_pressed() {
            if self.temperature <= MAX_TEMPERTURE_FOR_SHOOT
            {
                self.shoot(
                    player_id,
                    player,
                    screen_effects,
                    physic_system,
                    audio_system,
                    engine_handle,
                );
            }
        }

        let bar = match player.team {
            Team::Red => ui_system.get_mut_ui_element(&UIElementType::ShotgunBarRed),
            Team::Blue => ui_system.get_mut_ui_element(&UIElementType::ShotgunBarBlue),
        };

        if let UIElement::ProgressBar(bar) = bar {
            let value = {
                (self.temperature / MAX_TEMPERTURE)
                    .clamp(0.0, 1.0)
            };
            
            bar.set_bar_value(value)
        }

        self.cool_shotgun(delta);

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
            self.cool_shotgun(delta);

            let bar = match player.team {
                Team::Red => ui_system.get_mut_ui_element(&UIElementType::ShotgunBarRed),
                Team::Blue => ui_system.get_mut_ui_element(&UIElementType::ShotgunBarBlue),
            };

            if let UIElement::ProgressBar(bar) = bar {
                let value = {
                    (self.temperature / MAX_TEMPERTURE)
                        .clamp(0.0, 1.0)
                };
                
                bar.set_bar_value(value)
            }
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

        let bar = ui_system.get_mut_ui_element(&UIElementType::ShotgunBarRed);

        if let UIElement::ProgressBar(bar) = bar {
            bar.ui_data.is_visible = false;
        }

        let bar = ui_system.get_mut_ui_element(&UIElementType::ShotgunBarBlue);

        if let UIElement::ProgressBar(bar) = bar {
            bar.ui_data.is_visible = false;
        }
        let img = ui_system.get_mut_ui_element(&UIElementType::ShotgunImage);

        if let UIElement::Image(img) = img {
            img.ui_data.is_visible = false;
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
    )
    {
        let img = ui_system.get_mut_ui_element(&UIElementType::ShotgunImage);

        if let UIElement::Image(img) = img {
            img.ui_data.is_visible = true;
        }
    
        match player.team
        {
            Team::Red =>
            {
                let bar = ui_system.get_mut_ui_element(&UIElementType::ShotgunBarRed);

                if let UIElement::ProgressBar(bar) = bar {
                    bar.ui_data.is_visible = true;
                }

                let bar = ui_system.get_mut_ui_element(&UIElementType::ShotgunBarBlue);

                if let UIElement::ProgressBar(bar) = bar {
                    bar.ui_data.is_visible = false;
                }
            }

            Team::Blue =>
            {
                let bar = ui_system.get_mut_ui_element(&UIElementType::ShotgunBarBlue);

                if let UIElement::ProgressBar(bar) = bar {
                    bar.ui_data.is_visible = true;
                }

                let bar = ui_system.get_mut_ui_element(&UIElementType::ShotgunBarRed);

                if let UIElement::ProgressBar(bar) = bar {
                    bar.ui_data.is_visible = false;
                }
            }
        }
    }
}