pub mod player_input_master;
pub mod player_settings;

use crate::{
    actor::{
        device::{
            holegun::HoleGun, Device, DeviceType
        },
        Actor,
        ActorID,
        CommonActorsMessages,
        Component,
        Message,
        MessageType,
        SpecificActorMessage
    }, engine::{
        physics::{
            colliders_container::PhysicalElement,
            kinematic_collider::KinematicCollider,
            PhysicsSystem,
        },
        engine_handle::EngineHandle,
    },
    transform::Transform
};

use self::{
    player_input_master::InputMaster,
    player_settings::PlayerSettings,
};

use std::f32::consts::PI;
use glam::{Vec4, Mat4};

use super::holegun_hole::HoleGunHole;



pub struct PlayerInnerState {
    pub collider: KinematicCollider,
    pub transform: Transform,
    pub hp: i32,
}


impl PlayerInnerState {
    pub fn new(transform: Transform, settings: &PlayerSettings) -> Self {
        PlayerInnerState {
            collider: KinematicCollider::new(
                settings.max_speed,
                settings.max_accel,
                settings.collider_radius,
                settings.friction_on_air,
                // settings.friction_on_ground,
            ),
            transform,
            hp: 100,
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


pub struct Player {
    id: Option<ActorID>,

    inner_state: PlayerInnerState,

    view_angle: Vec4,

    active_hands_slot: ActiveHandsSlot, 

    hands_slot_0: Box<dyn Device>,
    hands_slot_1: Option<Box<dyn Device>>,
    hands_slot_2: Option<Box<dyn Device>>,
    hands_slot_3: Option<Box<dyn Device>>,

    devices: [Option<Box<dyn Device>>; 4],

    is_gravity_y_enabled: bool,
    is_gravity_w_enabled: bool,

    player_settings: PlayerSettings,

    no_collider_veclocity: Vec4,

    explore_w_position: f32,
    explore_w_coefficient: f32,

    pub master: InputMaster,
}


pub enum PLayerMessages {
    DealDamage(u32),
}


impl Actor for Player {
    fn recieve_message(&mut self, message: &Message, engine_handle: &mut EngineHandle) {
        let from = message.from;

        let message = &message.message;
        
        match message {
            MessageType::CommonActorsMessages(message) => {
                match message {
                    CommonActorsMessages::SetTransform(transform) => {
                        self.inner_state.transform = transform.clone();
                    },
                    CommonActorsMessages::EnableCollider(switch) => {
                        self.inner_state.collider.is_enable = *switch;
                    },
                    CommonActorsMessages::IncrementPosition(increment) => {
                        self.inner_state.transform.increment_position(increment.clone());
                    }
                }
            }
            MessageType::PhysicsMessages(message) => {
                match message {
                    _ => {}
                }
            },
            MessageType::SpecificActorMessage(message) => {
                match message {
                    SpecificActorMessage::PLayerMessages(message) => {
                        match message {
                            PLayerMessages::DealDamage(damage) => {
                                self.inner_state.hp -= *damage as i32;
                            }
                        }
                    },
                    _ => {},
                }

            }  
        }
    }


    fn get_mut_transform(&mut self) -> &mut Transform {
        &mut self.inner_state.transform
    }


    fn get_transform(&self) -> &Transform {
        &self.inner_state.transform
    }


    fn init(&mut self, id: ActorID) {
        self.id = Some(id);

        self.inner_state.collider.init(id);
    }


    fn get_id(&self) -> Option<ActorID> {
        self.id
    }


    fn get_physical_element(&mut self) -> Option<PhysicalElement> {
        let collider_container = PhysicalElement {
            transform: &mut self.inner_state.transform,
            kinematic_collider: Some(&mut self.inner_state.collider),
            static_colliders: None,
            static_objects: None,
            area: None,
        };

        Some(collider_container)
    }

    
    fn tick(
        &mut self,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        delta: f32
    ) {
        let my_id = self.id.expect("Player does not have id");

        let input = match &self.master {
            InputMaster::LocalMaster(master) => {
                master.current_input.clone()
            }
            InputMaster::RemoteMaster(master) => {
               master.current_input.clone()
            }   
        };

        let second_mouse_b_pressed = input.second_mouse.is_action_pressed();

        let mut x = self.view_angle.x;
        let mut y = self.view_angle.y;
        let mut xw = self.view_angle.z;
        let mut yw = self.view_angle.w;

        if second_mouse_b_pressed {
            xw = input.mouse_axis.x + xw;
            yw = (input.mouse_axis.y + yw).clamp(-PI/2.0, PI/2.0);
            
        } else {
            xw *= 1.0 - delta * 3.0;
            yw *= 1.0 - delta * 3.0;

            x = input.mouse_axis.x + x;
            y = (input.mouse_axis.y + y).clamp(-PI/2.0, PI/2.0);
        }


        let normal_rotation = Mat4::from_cols_slice(&[
            x.cos(),    y.sin() * x.sin(),  y.cos() * x.sin(),  0.0,
            0.0,        y.cos(),            -y.sin(),           0.0,
            -x.sin(),   y.sin() * x.cos(),  y.cos()*x.cos(),    0.0,
            0.0,        0.0,                0.0,                1.0
        ]);

        // let xw_rotation = Mat4::from_cols_slice(&[
        //     yw.cos(),    0.0,    0.0,    yw.sin(),
        //     0.0,        1.0,    0.0,    0.0,
        //     0.0,        0.0,    1.0,    0.0,
        //     -yw.sin(),   0.0,    0.0,    yw.cos()
        // ]);

        let yw_rotation = Mat4::from_cols_slice(&[
            1.0,    0.0,    0.0,        0.0,
            0.0,    1.0,    0.0,        0.0,
            0.0,    0.0,    yw.cos(),   yw.sin(),
            0.0,    0.0,    -yw.sin(),   yw.cos()
        ]);


        self.set_rotation_matrix(yw_rotation * normal_rotation);

        // self.set_rotation_matrix(Mat4::from_cols_slice(&[
        //     y.cos(),    0.0,    0.0,    y.sin(),
        //     0.0,        1.0,    0.0,    0.0,
        //     0.0,        0.0,    1.0,    0.0,
        //     -y.sin(),   0.0,    0.0,    y.cos()
        // ]));

        // self.set_rotation_matrix(Mat4::from_cols_slice(&[
        //     1.0,    0.0,        0.0,    0.0,
        //     0.0,    y.cos(),    0.0,    y.sin(),
        //     0.0,    0.0,        1.0,    0.0,
        //     0.0,    -y.sin(),   0.0,    y.cos()
        // ]));

        let xz_player_rotation = Mat4::from_rotation_y(x);
        self.view_angle = Vec4::new(x, y, xw, yw);  

        // self.inner_state.collision.transform.rotation *= new_rotation_matrix;

        match self.active_hands_slot {
            ActiveHandsSlot::Zero => {
                self.hands_slot_0.process_input(my_id, &mut self.inner_state, &input, physic_system, engine_handle);
            },
            ActiveHandsSlot::First => {
                if let Some(device) = self.hands_slot_1.as_mut() {
                    device.process_input(my_id, &mut self.inner_state, &input, physic_system, engine_handle);
                }
            },
            ActiveHandsSlot::Second => {
                if let Some(device) = self.hands_slot_2.as_mut() {
                    device.process_input(my_id, &mut self.inner_state, &input, physic_system, engine_handle);
                }
            },
            ActiveHandsSlot::Third => {
                if let Some(device) = self.hands_slot_3.as_mut() {
                    device.process_input(my_id, &mut self.inner_state, &input, physic_system, engine_handle);
                }
            }
        }

        for device in self.devices.iter_mut() {
            if let Some(device) = device {
                device.process_input(my_id, &mut self.inner_state, &input, physic_system, engine_handle);
            }
        }

        if input.mode_1.is_action_just_pressed() {
            self.is_gravity_y_enabled = !self.is_gravity_y_enabled;
        }

        if input.mode_2.is_action_just_pressed() {
            self.is_gravity_w_enabled = !self.is_gravity_w_enabled;
        }

        if input.mode_3.is_action_just_pressed() {
            self.inner_state.collider.is_enable = !self.inner_state.collider.is_enable;
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

        if input.jump.is_action_just_pressed() {

            if self.inner_state.collider.is_on_ground {
                self.inner_state.collider.add_force(Vec4::Y * self.player_settings.jump_y_speed);

            }
        }

        if input.w_up.is_action_pressed() {
            if self.inner_state.collider.is_enable {
                self.inner_state.collider.add_force(Vec4::W * self.player_settings.jetpak_w_speed);
            } else {
                self.no_collider_veclocity += Vec4::W * self.player_settings.jetpak_w_speed;
            }
        }

        if input.w_down.is_action_pressed() {
            if self.inner_state.collider.is_enable {
                self.inner_state.collider.add_force(Vec4::NEG_W * self.player_settings.jetpak_w_speed);
            } else {
                self.no_collider_veclocity += Vec4::NEG_W * self.player_settings.jetpak_w_speed;
            }
        }

        const MAX_EXPLORE_DIST: f32 = 2.5;
        const EXPLORE_SPEED: f32 = 0.7;

        if self.explore_w_position != 0.0 {
            if self.explore_w_position > 0.0 {
                if self.explore_w_position > MAX_EXPLORE_DIST {
                    self.explore_w_position = delta * -EXPLORE_SPEED;
                } else {
                    self.explore_w_position += delta * EXPLORE_SPEED;
                }
            } else {
                if self.explore_w_position < -MAX_EXPLORE_DIST {
                    self.explore_w_position = 0.0;
                } else {
                    self.explore_w_position -= delta * EXPLORE_SPEED;
                }
            }
        }

        if input.explore_w.is_action_just_pressed() {
            // self.inner_state.collider.add_force(Vec4::W * self.player_settings.jump_w_speed);
            // self.inner_state.collider.add_force(Vec4::Y * self.player_settings.jump_y_speed);

            if self.explore_w_position == 0.0 {
                self.explore_w_position = delta * self.player_settings.max_speed;
            }
        };

        self.explore_w_coefficient =
            (MAX_EXPLORE_DIST - self.explore_w_position.abs()) / MAX_EXPLORE_DIST;

        if self.inner_state.collider.is_enable {

            if self.is_gravity_y_enabled {
                movement_vec = self.get_rotation_matrix().inverse() * movement_vec;

                if self.inner_state.collider.is_on_ground {
                    self.inner_state.collider.set_wish_direction(
                        movement_vec,
                        1.0
                    );
                } else {
                    self.inner_state.collider.set_wish_direction(
                        movement_vec,
                        self.player_settings.air_speed_mult
                    );
                }

                self.inner_state.collider.add_force(Vec4::NEG_Y * self.player_settings.gravity_y_speed);

            } else {
               movement_vec = self.get_rotation_matrix().inverse() * movement_vec;

               self.inner_state.collider.set_wish_direction(movement_vec, 1.0);

            }

            if self.is_gravity_w_enabled {
                self.inner_state.collider.add_force(Vec4::NEG_W * self.player_settings.gravity_w_speed);
            }

        } else {
            
            movement_vec = self.get_rotation_matrix().inverse() * movement_vec;

            const MAX_SPEED: f32 = 24.0;
            const MAX_ACCEL: f32 = 32.0;

            if movement_vec.length().is_normal() {
                let current_speed_in_wishdir = self.no_collider_veclocity.dot(movement_vec);
    
                let speed = MAX_SPEED - current_speed_in_wishdir;
    
                let add_speed = 0.0_f32.max(speed.min(MAX_ACCEL * delta));
    
                self.no_collider_veclocity += movement_vec * add_speed;
    
            }

            self.inner_state.transform.increment_position(self.no_collider_veclocity * delta);
        }

        self.no_collider_veclocity *= 1.0 - delta*3.4;

        log::info!("Position: {:.2}", self.get_position());
    }
}



impl Player {

    pub fn new(master: InputMaster, player_settings: PlayerSettings) -> Self {
        Player {
            id: None,

            inner_state: PlayerInnerState::new(Transform::new_zero(), &player_settings),
            active_hands_slot: ActiveHandsSlot::Zero,

            hands_slot_0: Box::new(HoleGun{}),
            hands_slot_1: None,
            hands_slot_2: None,
            hands_slot_3: None,

            is_gravity_y_enabled: true,
            is_gravity_w_enabled: false,

            devices: [None, None, None, None],
            
            player_settings,

            master,

            explore_w_position: 0.0,
            explore_w_coefficient: 0.0,

            no_collider_veclocity: Vec4::ZERO,

            view_angle: Vec4::ZERO,
        }
    }

    pub fn get_explore_w_position(&self) -> f32 {
        self.explore_w_position
    }

    pub fn get_explore_w_coefficient(&self) -> f32 {
        self.explore_w_coefficient
    }

    pub fn get_position(&self) -> Vec4 {
        self.inner_state.transform.get_position()
    }


    pub fn get_rotation_matrix(&self) -> Mat4 {
        self.inner_state.transform.rotation.clone()
    }


    pub fn set_rotation_matrix(&mut self, new_rotation: Mat4) {
        self.inner_state.transform.rotation = new_rotation
    }


    pub fn get_collider_radius(&self) -> f32 {
        self.inner_state.collider.get_collider_radius()
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