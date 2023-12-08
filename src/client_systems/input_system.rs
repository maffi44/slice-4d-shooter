use std::collections::HashMap;

use winit::event::{KeyboardInput, VirtualKeyCode, ElementState, MouseButton};

use crate::common_systems::actions;

use actions::Action;
use actions::Actions;

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

        let mut actions = Actions::new();

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

    pub fn get_keyboard_input(&mut self, input: &KeyboardInput) {
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

    pub fn get_mouse_button_input(&mut self, button: &MouseButton, state: &ElementState) {
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