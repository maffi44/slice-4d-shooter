pub mod action;

use std::collections::HashMap;

use winit::{
    event::{
        ElementState,
         MouseButton,
         KeyEvent,
    },
    keyboard::{
        KeyCode,
        PhysicalKey,
    },
};

use super::{
    net::NetSystem,
    player::player_input_master::InputMaster::{
        LocalMaster,
        RemoteMaster,
    },
    world::World,
};

use action::Action;

use glam::Vec2;


const MOUSE_SENSITIVITY: f32 = 0.01;

#[derive(Clone)]
pub struct ActionsFrameState {
    pub move_forward: Action,
    pub move_backward: Action,
    pub move_right: Action,
    pub move_left: Action,
    pub jump: Action,
    pub fire: Action,
    pub mouse_axis: Vec2,
}

impl ActionsFrameState {
    fn current(actions_table: &HashMap<SomeButton, (ButtonActions, Action)>, mouse_axis: Vec2) -> Self {
        let mut move_forward = Action::new();
        let mut move_backward = Action::new();
        let mut move_right = Action::new();
        let mut move_left = Action::new();
        let mut jump = Action::new();
        let mut fire = Action::new();
        let mouse_axis = mouse_axis;
        
        for (_, (button_action, action)) in actions_table.iter() {
            match button_action {
                ButtonActions::MoveForward => move_forward = action.clone(),
                ButtonActions::MoveBackward => move_backward = action.clone(),
                ButtonActions::MoveRight => move_right = action.clone(),
                ButtonActions::MoveLeft => move_left = action.clone(),
                ButtonActions::Jump => jump = action.clone(),
                ButtonActions::Fire => fire = action.clone(),
            }
        }

        ActionsFrameState {
            move_forward,
            move_backward,
            move_right,
            move_left,
            jump,
            fire,
            mouse_axis
        }
    }

    pub fn empty() -> Self {
        let move_forward = Action::new();
        let move_backward = Action::new();
        let move_right = Action::new();
        let move_left = Action::new();
        let jump = Action::new();
        let fire = Action::new();
        let mouse_axis = Vec2::ZERO;

        ActionsFrameState {
            move_forward,
            move_backward,
            move_right,
            move_left,
            jump,
            fire,
            mouse_axis
        }
    }
}

pub enum MouseAxis {
    X,
    Y
}

#[derive(PartialEq, Eq, Hash)]
enum SomeButton {
    MouseButton(MouseButton),
    KeyCode(KeyCode),
}

enum ButtonActions {
    MoveForward,
    MoveBackward,
    MoveRight,
    MoveLeft,
    Jump,
    Fire,
}

// for future user's settings
enum ButtonActionsLinkedKeys {
    MoveForward(KeyCode),
    MoveBackward(KeyCode),
    MoveRight(KeyCode),
    MoveLeft(KeyCode),
    Jump(KeyCode),
    Fire(KeyCode),
}
pub struct InputSystem {
    actions_table: HashMap<SomeButton, (ButtonActions, Action)>,
    mouse_axis: Vec2,
}

impl InputSystem {

    pub fn new() -> Self {
        let mut actions_table = HashMap::new();

        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyW),
            (ButtonActions::MoveForward, Action::new())
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyS),
            (ButtonActions::MoveBackward, Action::new())
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyD),
            (ButtonActions::MoveRight, Action::new())
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyA),
            (ButtonActions::MoveLeft, Action::new())
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::Space),
            (ButtonActions::Jump, Action::new())
        );
        actions_table.insert(
            SomeButton::MouseButton(MouseButton::Left),
            (ButtonActions::Fire, Action::new())
        );
    
        InputSystem {
            actions_table,
            mouse_axis: Vec2::ZERO,
        }
    }

    pub fn get_input(&mut self, world: &mut World ,net: &mut NetSystem) {

        for (_, player) in world.pool_of_players.iter_mut() {
            match &mut player.master {
                LocalMaster(master) => {
                    master.current_input =
                        ActionsFrameState::current(
                            &self.actions_table,
                            self.mouse_axis
                        );
                    // log::info!("current input is {:?}", master.current_input);
                }
                RemoteMaster(master) => {
                    // Didn't implement yet
                }
            }
        }
    }




    pub fn reset_axis_input(&mut self) {
        self.mouse_axis = Vec2::ZERO;
    }

    pub fn add_mouse_delta(&mut self, delta: Vec2) {
        self.mouse_axis += delta * MOUSE_SENSITIVITY;
    }

    pub fn set_keyboard_input(&mut self, input: &KeyEvent) {

        if let PhysicalKey::Code(keycode) = input.physical_key {
    
            if let Some((_, action)) =
                self.actions_table.get_mut(&SomeButton::KeyCode(keycode)) {

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
                if let Some((_,action)) =
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
                if let Some((_,action)) =
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
                if let Some((_,action)) =
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
                if let Some((_,action)) =
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
            },
            MouseButton::Back => {},
            MouseButton::Forward => {},
        }
    }
}