use client_server_protocol::Team;
use glam::{FloatExt, Mat4, Vec4};

use crate::{actor::session_controller::DEFAULT_TEAM, engine::{audio::{AudioSystem, Sound}, engine_handle::EngineHandle, physics::{dynamic_collider::PlayersDollCollider, kinematic_collider::KinematicCollider, PhysicsSystem}, ui::{RectSize, UIElement, UIElementType, UISystem}}, transform::Transform};

use super::{player_settings::PlayerSettings, PlayerMovingState, PlayerScreenEffects, BASE_EFFECT_HP_IMPACT_SPEED, CROSSHAIR_DECREASING_SPEED, CROSSHAIR_INCREASING_SPEED, CROSSHAIR_MAX_SIZE, CROSSHAIR_MIN_SIZE, PLAYER_MAX_HP};

pub struct PlayerInnerState {
    pub team: Team,
    pub collider: KinematicCollider,
    pub collider_for_others: Vec<PlayersDollCollider>,
    pub transform: Transform,
    pub hp: f32,
    pub is_alive: bool,
    pub is_enable: bool,
    
    pub crosshair_target_size: f32,
    pub crosshair_size: f32,

    pub zw_rotation: Mat4,
    pub zy_rotation: Mat4,
    pub zx_rotation: Mat4,

    pub is_time_after_some_team_win: bool,
    pub amount_of_move_w_bonuses_do_i_have: u32,
    // pub player_moving_state: PlayerMovingState,

    pub blue_map_w_level: f32,
    pub red_map_w_level: f32,

    pub friction_on_air: f32,

    pub screen_effects: PlayerScreenEffects,
    
    pub show_crosshaier_hit_mark_timer: f32,
    pub last_frame_zw_rotation: f32,

    pub jumped_to_y_on_current_action: bool,
    pub player_previous_w_position: f32,
    
    pub saved_angle_of_rotation: f32,
}


impl PlayerInnerState {
    pub fn new(
        transform: Transform,
        settings: &PlayerSettings,
        is_alive: bool,
        is_enable: bool,
        blue_map_w_level: f32,
        red_map_w_level: f32,
    ) -> Self {

        let collider_for_others = {
            let mut vec = Vec::with_capacity(1);
            
            vec.push(PlayersDollCollider {
                position: Vec4::ZERO,
                radius: settings.collider_radius,
                friction: 0_f32,
                bounce_rate: 0_f32,
                actors_id: None,
                weapon_offset: Vec4::Y*0.6,
                actors_team: DEFAULT_TEAM,
            });
            vec
        };

        PlayerInnerState {
            team: DEFAULT_TEAM,
            collider: KinematicCollider::new(
                settings.max_speed,
                settings.max_accel,
                settings.collider_radius,
                settings.friction_on_air,
                // settings.friction_on_ground,
            ),
            collider_for_others,
            transform,
            hp: 0.0,
            is_alive,
            is_enable,
            crosshair_target_size: 0.04,
            crosshair_size: 0.04,
            show_crosshaier_hit_mark_timer: 0.0,

            zw_rotation: Mat4::IDENTITY,
            zy_rotation: Mat4::IDENTITY,
            zx_rotation: Mat4::IDENTITY,

            is_time_after_some_team_win: false,
            amount_of_move_w_bonuses_do_i_have: 0u32,
            // player_moving_state: PlayerMovingState::MovingPerpendicularW(0.0),

            blue_map_w_level,
            red_map_w_level,
            friction_on_air: settings.friction_on_air,
            last_frame_zw_rotation: 0.0,

            jumped_to_y_on_current_action: false,
            player_previous_w_position: 0.0,
            saved_angle_of_rotation: 0.0,
            screen_effects: PlayerScreenEffects::default(),
        }
    }

    pub fn get_eyes_offset(&self) -> Vec4
    {
        Vec4::Y * self.collider.get_collider_radius() * 0.2
    }

    pub fn get_eyes_position(&self) -> Vec4
    {
        self.transform.get_position() + self.get_eyes_offset()
    }

    pub fn get_position(&self) -> Vec4 {
        self.transform.get_position()
    }

    pub fn get_rotation_matrix(&self) -> Mat4 {
        self.transform.get_rotation()
    }

    pub fn get_zw_rotation_matrix(&self) -> Mat4 {
        self.zw_rotation
    }

    pub fn get_zy_rotation_matrix(&self) -> Mat4 {
        self.zy_rotation
    }

    pub fn get_zx_rotation_matrix(&self) -> Mat4 {
        self.zx_rotation
    }

    pub fn get_team(&self) -> Team
    {
        self.team
    }

    pub fn set_rotation_matrix(&mut self, new_rotation: Mat4) {
        self.transform.set_rotation(new_rotation)
    }

    pub fn get_collider_radius(&self) -> f32 {
        self.collider.get_collider_radius()
    }

    pub fn process_crosshair_size_and_ui(&mut self, ui_system: &mut UISystem, delta: f32)
    {
        let crosshair = ui_system.get_mut_ui_element(&UIElementType::Crosshair);

        if let UIElement::Image(crosshair) = crosshair {
            crosshair.ui_data.rect.size = RectSize::LockedHeight(self.crosshair_size);
        }

        self.crosshair_target_size = self.crosshair_target_size
            .min(CROSSHAIR_MAX_SIZE); 

        if self.crosshair_size < self.crosshair_target_size {

            self.crosshair_size += CROSSHAIR_INCREASING_SPEED*delta;

            if self.crosshair_size >= self.crosshair_target_size {
                self.crosshair_size = self.crosshair_target_size;
                
                self.crosshair_target_size = CROSSHAIR_MIN_SIZE;
            }
        } else {
            self.crosshair_size =
                (self.crosshair_size - CROSSHAIR_DECREASING_SPEED*delta)
                .max(CROSSHAIR_MIN_SIZE);
        }

        if self.show_crosshaier_hit_mark_timer > 0.0
        {
            let crosshair_hit_mark = ui_system.get_mut_ui_element(&UIElementType::CrosshairHitMark);
    
            *crosshair_hit_mark.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = true;

            self.show_crosshaier_hit_mark_timer -= delta;
        }
        else
        {
            let crosshair_hit_mark = ui_system.get_mut_ui_element(&UIElementType::CrosshairHitMark);
    
            *crosshair_hit_mark.get_ui_data_mut().get_is_visible_cloned_arc().lock().unwrap() = false;
        }
    }
}