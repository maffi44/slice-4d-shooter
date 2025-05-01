use glam::{Vec3, Vec4};

use crate::{
    actor::{
        machinegun_shot::MachinegunShot, main_player::{
            player_inner_state::PlayerInnerState,
            PlayerMessage,
            PlayerScreenEffects,
        }, shotgun_shot_source::ShotgunShotSource, ActorID, ActorWrapper, Message, MessageType, SpecificActorMessage
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
        render::VisualElement,
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


const FIRE_RATE: f32 = 0.11;
const MAX_TEMPERTURE: f32 = 60.0;
const MAX_SHOOTING_RANGE: f32 = 0.0023;
const SHOOTING_RANGE_INCR_SPEED: f32 = 15.0;
const SHOOTING_RANGE_DCR_SPEED: f32 = 15.0;
const CROSSHAIR_INCREASE_ON_SHOOT: f32 = 0.2;

pub const SHOTGUN_LASER_SHOT_HOLE_REDUCTION_SPEED: f32 = 0.35;
pub const SHOTGUN_LASER_SHOT_EXPLOSION_EXPAND_SPEED: f32 = 6.2;
pub const SHOTGUN_LASER_SHOT_EXPLOSION_MAX_RADIUS: f32 = 0.25;
pub const SHOTGUN_LASER_SHOT_EXPLOSION_HOLE_MULT: f32 = 0.4;
pub const SHOTGUN_LASER_SHOT_MAX_DISTANCE: f32 = 200.0;
pub const SHOTGUN_LASER_SHOT_DAMAGE: u32 = 10;
pub const SHOTGUN_LASER_SHOT_ADD_FORCE_PER_HIT: f32 = 2.0;
pub const SHOTGUN_LASER_SHOT_SPEED: f32 = 155.5;
pub const SHOTGUN_LASER_SHOT_LENGTH: f32 = 7.6;
pub const SHOTGUN_LASER_SHOT_BEAM_RADIUS: f32 = 0.045;
pub const SHOTGUN_LASER_SHOT_COLOR: Vec3 = Vec3::new(1.0, 0.3, 0.0);
pub const SHOTGUN_SHOT_FLASH_EXPLAND_SPEED: f32 = 2.5;
pub const SHOTGUN_SHOT_FLASH_MAX_RADIUS: f32 = 0.12;
pub const SHOTGUN_SHOT_FLASH_FADE_SPEED: f32 = 2.5;

pub const SHOTGUN_LASER_SHOTS_AMOUNT: u32 = 18;
pub const SHOTGUN_LASER_SHOTS_AMOUNT_WITHOUT_W_SPREAD: u32 = 4;
pub const SHOTGUN_SHOTS_SPREAD: f32 = 0.175;

pub struct Shotgun {
    // temperature: f32,
    // shooting_range: f32,
    // time_from_prev_shot: f32,
    // is_overheating: bool,

    shooted_from_pivot_point_dir: Vec4,

    // machinegun_damage: f32,
    // machinegun_add_force: f32, 
    // machinegun_heat_add_on_shot: f32, 
    // machinegun_cooling_speed: f32
}

impl Shotgun {
    pub fn new(
        // machinegun_damage: f32,
        // machinegun_add_force: f32, 
        // machinegun_heat_add_on_shot: f32, 
        // machinegun_cooling_speed: f32,
        shooted_from_pivot_point_dir: Vec4,

    ) -> Self {

        Shotgun {
            // temperature: 0.0,
            // shooting_range: 0.0,
            // time_from_prev_shot: 0.0,
            // is_overheating: false,
            shooted_from_pivot_point_dir,

            // machinegun_damage,
            // machinegun_add_force, 
            // machinegun_heat_add_on_shot, 
            // machinegun_cooling_speed
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
            engine_handle,
            physic_system,
        );


        engine_handle.send_command(
            Command {
                sender: player_id,
                command_type: CommandType::SpawnActor(
                    ActorWrapper::ShotgunShotSource(shotgun_shot_source)
                )
            }
        );

        // engine_handle.send_command(
        //     Command {
        //         sender: player_id,
        //         command_type: CommandType::NetCommand(
        //             NetCommand::SendBoardcastNetMessageReliable(
        //                 NetMessageToPlayer::RemoteDirectMessage(
        //                     player_id,
        //                     RemoteMessage::SpawnMachineGunShot(
        //                         position.to_array(),
        //                         false,
        //                     )
        //                 )
        //             )
        //         )
        //     }
        // )
    }
}

impl Device for Shotgun {
    fn get_device_type(&self) -> DeviceType {
        DeviceType::Gun
    }

    fn get_visual_element<'a>(&'a self, transform: &'a Transform) -> Option<VisualElement<'a>> {
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
        if input.first_mouse.is_action_just_pressed() {
            self.shoot(
                player_id,
                player,
                screen_effects,
                physic_system,
                audio_system,
                engine_handle,
            );
        }

        // let bar = match player.team {
        //     Team::Red => ui_system.get_mut_ui_element(&UIElementType::MachinegunBarRed),
        //     Team::Blue => ui_system.get_mut_ui_element(&UIElementType::MachinegunBarBlue),
        // };

        // if let UIElement::ProgressBar(bar) = bar {
        //     let value = {
        //         (self.temperature / MAX_TEMPERTURE)
        //             .clamp(0.0, 1.0)
        //     };
            
        //     bar.set_bar_value(value)
        // }
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
            // self.cool_machinegun(delta);
            // self.time_from_prev_shot += delta;

            // let bar = match player.team {
            //     Team::Red => ui_system.get_mut_ui_element(&UIElementType::MachinegunBarRed),
            //     Team::Blue => ui_system.get_mut_ui_element(&UIElementType::MachinegunBarBlue),
            // };

            // if let UIElement::ProgressBar(bar) = bar {
            //     let value = {
            //         (self.temperature / MAX_TEMPERTURE)
            //             .clamp(0.0, 1.0)
            //     };
                
            //     bar.set_bar_value(value)
            // }
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

            // let bar = ui_system.get_mut_ui_element(&UIElementType::MachinegunBarRed);

            // if let UIElement::ProgressBar(bar) = bar {
            //     *bar.ui_data.is_visible.lock().unwrap() = false;
            // }
    
            // let bar = ui_system.get_mut_ui_element(&UIElementType::MachinegunBarBlue);
    
            // if let UIElement::ProgressBar(bar) = bar {
            //     *bar.ui_data.is_visible.lock().unwrap() = false;
            // }
    
            // let img = ui_system.get_mut_ui_element(&UIElementType::MachinegunImage);
    
            // if let UIElement::Image(img) = img {
            //     *img.ui_data.is_visible.lock().unwrap() = false;
            // }
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
        // let img = ui_system.get_mut_ui_element(&UIElementType::MachinegunImage);

        // if let UIElement::Image(img) = img {
        //     *img.ui_data.is_visible.lock().unwrap() = true;
        // }

        // match player.team
        // {
        //     Team::Red =>
        //     {
        //         let bar = ui_system.get_mut_ui_element(&UIElementType::MachinegunBarRed);

        //         if let UIElement::ProgressBar(bar) = bar {
        //             *bar.ui_data.is_visible.lock().unwrap() = true;
        //         }

        //         let bar = ui_system.get_mut_ui_element(&UIElementType::MachinegunBarBlue);

        //         if let UIElement::ProgressBar(bar) = bar {
        //             *bar.ui_data.is_visible.lock().unwrap() = false;
        //         }
        //     }

        //     Team::Blue =>
        //     {
        //         let bar = ui_system.get_mut_ui_element(&UIElementType::MachinegunBarBlue);

        //         if let UIElement::ProgressBar(bar) = bar {
        //             *bar.ui_data.is_visible.lock().unwrap() = true;
        //         }

        //         let bar = ui_system.get_mut_ui_element(&UIElementType::MachinegunBarRed);

        //         if let UIElement::ProgressBar(bar) = bar {
        //             *bar.ui_data.is_visible.lock().unwrap() = false;
        //         }
        //     }
        // }
    }
}