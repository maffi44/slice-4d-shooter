pub mod player_input_master;
pub mod player_settings;

use std::f32::consts::PI;

use glam::{
    Vec2,
    Vec4,
    Mat4,
};

use player_input_master::InputMaster;
use player_settings::PlayerSettings;

use super::{
    devices::{Device, DeviceType, DefaultPistol},
    engine_handle::EngineHandle,
    transform::Transform,
    physics::collisions::DynamicCollision,
};

pub type PlayerID = u32;
pub struct PlayerInnerState {
    pub collision: DynamicCollision,
    pub hp: i32,
    pub view_angle: Vec2,
}

impl PlayerInnerState {
    pub fn new(transform: Transform, settings: &PlayerSettings) -> Self {
        PlayerInnerState {
            collision: DynamicCollision::new(
                transform,
                settings.max_speed,
                settings.max_accel,
                settings.collider_radius,
                settings.friction_on_air,
                settings.friction_on_ground,
            ),
            hp: 100,
            view_angle: Vec2::ZERO,
        }
    }
}

enum ActiveHandsSlot {
    Zero,
    First,
    Second,
    Third,
}

pub enum PlayersDeviceSlotNumber {
    First,
    Second,
    Third,
    Fourth,
}

pub enum Message {
    DealDamage(u32),
    SetTransform(Transform),
    EnableCollider(bool)
}

pub struct Player {
    id: PlayerID,

    inner_state: PlayerInnerState,

    active_hands_slot: ActiveHandsSlot, 

    hands_slot_0: Box<dyn Device>,
    hands_slot_1: Option<Box<dyn Device>>,
    hands_slot_2: Option<Box<dyn Device>>,
    hands_slot_3: Option<Box<dyn Device>>,

    devices: [Option<Box<dyn Device>>; 4],

    is_gravity_y_enabled: bool,
    is_gravity_w_enabled: bool,

    player_settings: PlayerSettings,

    pub master: InputMaster,
}

impl Player {

    pub fn new(id: PlayerID, master: InputMaster, player_settings: PlayerSettings) -> Self {
        Player {
            id,

            inner_state: PlayerInnerState::new(Transform::new_zero(), &player_settings),
            active_hands_slot: ActiveHandsSlot::Zero,

            hands_slot_0: Box::new(DefaultPistol::default()),
            hands_slot_1: None,
            hands_slot_2: None,
            hands_slot_3: None,

            is_gravity_y_enabled: true,
            is_gravity_w_enabled: false,

            devices: [None, None, None, None],
            
            player_settings,

            master,
        }
    }


    pub fn get_collider(&self) -> &DynamicCollision {
        &self.inner_state.collision
    }

    pub fn get_mut_collider(&mut self) -> &mut DynamicCollision {
        &mut self.inner_state.collision
    }

    pub fn get_position(&self) -> Vec4 {
        self.get_collider().transform.get_position()
    }

    pub fn get_rotation_matrix(&self) -> Mat4 {
        self.inner_state.collision.transform.rotation.clone()
    }

    pub fn set_rotation_matrix(&mut self, new_rotation: Mat4) {
        self.inner_state.collision.transform.rotation = new_rotation
    }

    pub fn recieve_message(&mut self, from: PlayerID, message: Message, engine_handle: &mut EngineHandle) {
        match message {
            Message::DealDamage(damage) => {
                self.inner_state.hp -= damage as i32;
            },
            Message::SetTransform(transform) => {
                self.inner_state.collision.transform = transform;
            }
            Message::EnableCollider(enable) => {
                self.get_mut_collider().is_enable = enable;
            }
        }
    }

    pub fn process_input(&mut self, engine_handle: &mut EngineHandle) {

        let input = match &self.master {
            InputMaster::LocalMaster(master) => {
                master.current_input.clone()
            }
            InputMaster::RemoteMaster(master) => {
               master.current_input.clone()
            }   
        };

        let prev_x = self.inner_state.view_angle.x;
        let prev_y = self.inner_state.view_angle.y;

        let x = input.mouse_axis.x + prev_x;
        let y = input.mouse_axis.y + prev_y.clamp(-PI/2.0, PI/2.0);

        self.set_rotation_matrix(Mat4::from_cols_slice(&[
            x.cos(),    y.sin() * x.sin(),  y.cos() * x.sin(),  0.0,
            0.0,        y.cos(),            -y.sin(),           0.0,
            -x.sin(),   y.sin() * x.cos(),  y.cos()*x.cos(),    0.0,
            0.0,        0.0,                0.0,                1.0
        ]));

        let xz_player_rotation = Mat4::from_rotation_y(x);

        self.inner_state.view_angle = Vec2::new(x, y);

        // self.inner_state.collision.transform.rotation *= new_rotation_matrix;

        match self.active_hands_slot {
            ActiveHandsSlot::Zero => {
                self.hands_slot_0.process_input(self.id, &mut self.inner_state, &input, engine_handle);
            },
            ActiveHandsSlot::First => {
                if let Some(device) = self.hands_slot_1.as_mut() {
                    device.process_input(self.id, &mut self.inner_state, &input, engine_handle);
                }
            },
            ActiveHandsSlot::Second => {
                if let Some(device) = self.hands_slot_2.as_mut() {
                    device.process_input(self.id, &mut self.inner_state, &input, engine_handle);
                }
            },
            ActiveHandsSlot::Third => {
                if let Some(device) = self.hands_slot_3.as_mut() {
                    device.process_input(self.id, &mut self.inner_state, &input, engine_handle);
                }
            }
        }

        for device in self.devices.iter_mut() {
            if let Some(device) = device {
                device.process_input(self.id, &mut self.inner_state, &input, engine_handle);
            }
        }


        let mut movement_vec = Vec4::ZERO;

        if input.move_forward.is_action_pressed() {
            movement_vec += Vec4::NEG_Z;
        }

        if input.move_backward.is_action_pressed() {
            movement_vec += Vec4::Z;
        }

        if input.move_right.is_action_pressed() {
            movement_vec += Vec4::X;
        }

        if input.move_left.is_action_pressed() {
            movement_vec += Vec4::NEG_X;
        }

        if let Some(vec) = movement_vec.try_normalize() {
            movement_vec = vec;
        }

        if input.mode_1.is_action_just_pressed {
            self.is_gravity_y_enabled = !self.is_gravity_y_enabled;
        }

        if input.mode_2.is_action_just_pressed {
            self.is_gravity_w_enabled = !self.is_gravity_w_enabled;
        }

        if input.mode_3.is_action_just_pressed {
            self.get_mut_collider().is_enable = !self.get_mut_collider().is_enable;
        }

        if self.get_collider().is_enable {

            if self.is_gravity_y_enabled {
                movement_vec = xz_player_rotation * movement_vec;

                if self.get_collider().is_on_ground {
                    self.inner_state.collision.set_wish_direction(
                        movement_vec,
                        1.0
                    );
                } else {
                    self.inner_state.collision.set_wish_direction(
                        movement_vec,
                        self.player_settings.air_speed_mult
                    );
                }

                self.inner_state.collision.add_force(Vec4::NEG_Y * self.player_settings.gravity_y_speed);
            } else {
               movement_vec = self.get_rotation_matrix().inverse() * movement_vec;

               self.inner_state.collision.set_wish_direction(movement_vec, 1.0);

            }

            if self.is_gravity_w_enabled {
                self.inner_state.collision.add_force(Vec4::NEG_W * self.player_settings.gravity_w_speed);
            }

        } else {
            movement_vec = self.get_rotation_matrix().inverse() * movement_vec;

            self.inner_state.collision.set_wish_direction(movement_vec, 1.0);
        }

        if input.jump.is_action_just_pressed() {
            self.inner_state.collision.add_force(Vec4::Y * self.player_settings.jump_y_speed);
            if self.get_collider().is_on_ground {

            }
        }


        if input.w_up.is_action_pressed() {
            self.inner_state.collision.add_force(Vec4::W * self.player_settings.jetpak_w_speed);
        }

        if input.w_down.is_action_pressed() {
            self.inner_state.collision.add_force(Vec4::NEG_W * self.player_settings.jetpak_w_speed);
        }

        if input.crouch.is_action_just_pressed() {
            self.inner_state.collision.add_force(Vec4::W * self.player_settings.jump_w_speed);
            self.inner_state.collision.add_force(Vec4::Y * self.player_settings.jump_y_speed);
        };

    }

    pub fn set_gun_to_1_slot(
        &mut self,
        device: Box<dyn Device>
    ) -> Option<Box<dyn Device>>
    {

        match device.get_device_type() {
            DeviceType::Gun => {
                let prev_device = self.hands_slot_1.take();
                self.hands_slot_1 = Some(device);

                return prev_device;
            }
            _ => {
                Some(device)
            }
        }
    }

    pub fn set_gun_to_2_slot(
        &mut self,
        device: Box<dyn Device>
    ) -> Option<Box<dyn Device>>
    {

        match device.get_device_type() {
            DeviceType::Gun => {
                let prev_device = self.hands_slot_2.take();
                self.hands_slot_2 = Some(device);

                return prev_device;
            }
            _ => {
                Some(device)
            }
        }
    }

    pub fn set_gun_to_3_slot(
        &mut self,
        device: Box<dyn Device>
    ) -> Option<Box<dyn Device>>
    {

        match device.get_device_type() {
            DeviceType::Gun => {
                let prev_device = self.hands_slot_3.take();
                self.hands_slot_3 = Some(device);

                return prev_device;
            }
            _ => {
                Some(device)
            }
        }
    }

    pub fn set_device_to_device_slot(
        &mut self,
        slot_number: PlayersDeviceSlotNumber,
        device: Box<dyn Device>
    ) -> Option<Box<dyn Device>> {

        match device.get_device_type() {
            DeviceType::Device => {
                match slot_number {
                    PlayersDeviceSlotNumber::First => {
                        let prev_device = self.devices[0].take();
                        self.devices[0] = Some(device);
                        prev_device
                    }
                    PlayersDeviceSlotNumber::Second => {
                        let prev_device = self.devices[1].take();
                        self.devices[1] = Some(device);
                        prev_device
                    }
                    PlayersDeviceSlotNumber::Third => {
                        let prev_device = self.devices[2].take();
                        self.devices[2] = Some(device);
                        prev_device
                    }
                    PlayersDeviceSlotNumber::Fourth => {
                        let prev_device = self.devices[3].take();
                        self.devices[3] = Some(device);
                        prev_device
                    }
                }

                
            },
            _ => {Some(device)}
        }
    }
}