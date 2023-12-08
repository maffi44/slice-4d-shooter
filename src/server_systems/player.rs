use glam::Vec4;

const MAX_SPEED : f32 = 100.0;
const MAX_ACCEL : f32 = 100.0;
const HEALTH: i32 = 100_i32;

use super::{
    devices::{Device, DeviceType, DefaultPistol},
    engine_handle::EngineHandle,
};

use crate::common_systems::{
    actions::Actions,
    transform::{Transform, self}, physics_system::collisions::{Collision, DynamicCollision},
};

pub type PlayerID = u32;
pub struct PlayerInnerState {
    pub collision: DynamicCollision,
    pub hp: i32,
}

impl PlayerInnerState {
    pub fn new(transform: Transform) -> Self {
        PlayerInnerState {
            collision: DynamicCollision::new(
                transform,
                MAX_SPEED,
                MAX_ACCEL,
            ),
            hp: HEALTH,
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
}

impl Player {

    pub fn new(id: PlayerID) -> Self {
        Player {
            id,

            inner_state: PlayerInnerState::new(Transform::new_zero()),
            active_hands_slot: ActiveHandsSlot::Zero,

            hands_slot_0: Box::new(DefaultPistol::default()),
            hands_slot_1: None,
            hands_slot_2: None,
            hands_slot_3: None,

            devices: [None, None, None, None]
        }
    }

    pub fn recieve_message(&mut self, from: PlayerID, message: Message, engine_handle: &mut EngineHandle) {
        match message {
            Message::DealDamage(damage) => {
                self.inner_state.hp -= damage as i32;
            },
        }
    }

    pub fn process_input(&mut self, input: &mut Actions, engine_handle: &mut EngineHandle) {

        match self.active_hands_slot {
            ActiveHandsSlot::Zero => {
                self.hands_slot_0.process_input(self.id, &mut self.inner_state, input, engine_handle);
            },
            ActiveHandsSlot::First => {
                if let Some(device) = self.hands_slot_1.as_mut() {
                    device.process_input(self.id, &mut self.inner_state, input, engine_handle);
                }
            },
            ActiveHandsSlot::Second => {
                if let Some(device) = self.hands_slot_2.as_mut() {
                    device.process_input(self.id, &mut self.inner_state, input, engine_handle);
                }
            },
            ActiveHandsSlot::Third => {
                if let Some(device) = self.hands_slot_3.as_mut() {
                    device.process_input(self.id, &mut self.inner_state, input, engine_handle);
                }
            }
        }

        for device in self.devices.iter_mut() {
            if let Some(device) = device {
                device.process_input(self.id, &mut self.inner_state, input, engine_handle);
            }
        }


        // process movement
        let mut movement_vec = Vec4::ZERO;

        if input.move_forward.is_action_pressed() {
            input.move_forward.capture_action();

            movement_vec += Vec4::new(0.0, 0.0, 1.0, 0.0);
        }

        if input.move_backward.is_action_pressed() {
            input.move_backward.capture_action();

            movement_vec += Vec4::new(0.0, 0.0, -1.0, 0.0);
        }

        if input.move_right.is_action_pressed() {
            input.move_right.capture_action();

            movement_vec += Vec4::new(1.0, 0.0, 0.0, 0.0);
        }

        if input.move_left.is_action_pressed() {
            input.move_left.capture_action();

            movement_vec += Vec4::new(-1.0, 0.0, 0.0, 0.0);
        }

        

        self.inner_state.collision.add_wish_direction(movement_vec)

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