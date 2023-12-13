use std::collections::HashMap;

use winit::event::{KeyboardInput, VirtualKeyCode, ElementState, MouseButton};

use super::{
    actions,
    net::NetSystem,
    player_input_master::InputMaster::{
        LocalMaster,
        RemoteMaster,
    },
    world::World,
};

use actions::Action;
use actions::Actions;

pub enum MouseAxis {
    X,
    Y
}

pub struct InputSystem {
    actions_table: HashMap<SomeButton, &'static mut Action>,
    pub actions: Actions,
}


#[derive(PartialEq, Eq, Hash)]
enum SomeButton {
    MouseButton(MouseButton),
    VirtualKeyCode(VirtualKeyCode),
}


impl InputSystem {

    pub fn new() -> Self {
        let mut actions_table = HashMap::new();

        let actions = Actions::new();

        unsafe {
            actions_table.insert(
                SomeButton::VirtualKeyCode(VirtualKeyCode::W),
                &mut actions::MOVE_FORWARD
            );
            actions_table.insert(
                SomeButton::VirtualKeyCode(VirtualKeyCode::S),
                &mut actions::MOVE_BACKWARD
            );
            actions_table.insert(
                SomeButton::VirtualKeyCode(VirtualKeyCode::D),
                &mut actions::MOVE_RIGHT
            );
            actions_table.insert(
                SomeButton::VirtualKeyCode(VirtualKeyCode::A),
                &mut actions::MOVE_LEFT
            );
            actions_table.insert(
                SomeButton::VirtualKeyCode(VirtualKeyCode::Space),
                &mut actions::JUMP
            );
            actions_table.insert(
                SomeButton::MouseButton(MouseButton::Left),
                &mut actions::FIRE
            );
        }
        
        InputSystem {actions_table, actions}
    }

    pub fn get_input(&mut self, world: &mut World ,net: &mut NetSystem) {

        for (_, player) in world.pool_of_players.iter_mut() {
            match &mut player.master {
                LocalMaster(master) => {
                    master.current_input = self.actions.clone();
                }
                RemoteMaster(master) => {
                    // Didn't implement yet
                }
            }
        }
    }


    pub fn reset_axis_input(&mut self) {
        self.actions.axis_input = glam::Vec2::ZERO;
    }

    pub fn add_axis_motion(&mut self, axis: MouseAxis, value: f64) {
        match axis {
            MouseAxis::X => {
                self.actions.axis_input.x += value as f32;
            }
            MouseAxis::Y => {
                self.actions.axis_input.y += value as f32;
            }
        }
    }

    pub fn set_keyboard_input(&mut self, input: &KeyboardInput) {
        if let Some(keycode) = input.virtual_keycode {
            if let Some(action) =
                self.actions_table.get_mut(&SomeButton::VirtualKeyCode(keycode)) {

                match input.state {
                    ElementState::Pressed => {
                        if action.is_action_pressed == false {
                            action.is_action_just_pressed = true;
                            action.is_action_pressed = true;
                        } else {
                            action.is_action_just_pressed = false;
                            action.is_action_pressed = true;
                        }
                    },
                    ElementState::Released => {
                        action.is_action_just_pressed = false;
                        action.is_action_pressed = false;
                    }
                }
                // action.already_captured = false;
            }
        }
    }

    pub fn set_mouse_button_input(&mut self, button: &MouseButton, state: &ElementState) {
        match button {
            MouseButton::Left => {
                if let Some(action) =
                    self.actions_table.get_mut(&SomeButton::MouseButton(MouseButton::Left)) {
                    
                    match state {
                        ElementState::Pressed => {
                            if action.is_action_pressed == false {
                                action.is_action_just_pressed = true;
                                action.is_action_pressed = true;
                            } else {
                                action.is_action_just_pressed = false;
                                action.is_action_pressed = true;
                            }
                        },
                        ElementState::Released => {
                            action.is_action_just_pressed = false;
                            action.is_action_pressed = false;
                        },
                    }
                    // action.already_captured = false;
                }
            },
            MouseButton::Middle => {
                if let Some(action) =
                    self.actions_table.get_mut(&SomeButton::MouseButton(MouseButton::Middle)) {
                    
                    match state {
                        ElementState::Pressed => {
                            if action.is_action_pressed == false {
                                action.is_action_just_pressed = true;
                                action.is_action_pressed = true;
                            } else {
                                action.is_action_just_pressed = false;
                                action.is_action_pressed = true;
                            }
                        },
                        ElementState::Released => {
                            action.is_action_just_pressed = false;
                            action.is_action_pressed = false;
                        },
                    }
                    // action.already_captured = false;
                }
            },
            MouseButton::Right => {
                if let Some(action) =
                    self.actions_table.get_mut(&SomeButton::MouseButton(MouseButton::Right)) {
                    
                    match state {
                        ElementState::Pressed => {
                            if action.is_action_pressed == false {
                                action.is_action_just_pressed = true;
                                action.is_action_pressed = true;
                            } else {
                                action.is_action_just_pressed = false;
                                action.is_action_pressed = true;
                            }
                        },
                        ElementState::Released => {
                            action.is_action_just_pressed = false;
                            action.is_action_pressed = false;
                        },
                    }
                    // action.already_captured = false;
                }
            },
            MouseButton::Other(code) => {
                if let Some(action) =
                    self.actions_table.get_mut(&SomeButton::MouseButton(MouseButton::Other(*code))) {
                    
                    match state {
                        ElementState::Pressed => {
                            if action.is_action_pressed == false {
                                action.is_action_just_pressed = true;
                                action.is_action_pressed = true;
                            } else {
                                action.is_action_just_pressed = false;
                                action.is_action_pressed = true;
                            }
                        },
                        ElementState::Released => {
                            action.is_action_just_pressed = false;
                            action.is_action_pressed = false;
                        },
                    }
                    // action.already_captured = false;
                }
            }
        }
    }
}