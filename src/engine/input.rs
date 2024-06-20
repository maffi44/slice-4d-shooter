pub mod action;

use crate::{
    engine::{
        net::NetSystem,
        world::World,
    },
    actor::{
        player::player_input_master::InputMaster::{
            LocalMaster,
            RemoteMaster,
        },
        ActorWrapper,
    },
};

use self::action::Action;

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
use glam::Vec2;



const MOUSE_SENSITIVITY: f32 = 0.003;

#[derive(Clone)]
pub struct ActionsFrameState {
    pub move_forward: Action,
    pub move_backward: Action,
    pub move_right: Action,
    pub move_left: Action,
    pub w_down: Action,
    pub w_up: Action,
    pub w_scanner: Action,
    pub activate_hand_slot_0: Action,
    pub activate_hand_slot_1: Action,
    pub activate_hand_slot_2: Action,
    pub activate_hand_slot_3: Action,
    pub mode_1: Action,
    pub mode_2: Action,
    pub mode_3: Action,
    pub jump: Action,
    pub jump_w: Action,
    pub first_mouse: Action,
    pub second_mouse: Action,
    pub mouse_axis: Vec2,
}

impl ActionsFrameState {
    fn current(actions_table: &HashMap<SomeButton, (ButtonActions, Action)>, mouse_axis: Vec2) -> Self {
        let mut move_forward = Action::new();
        let mut move_backward = Action::new();
        let mut move_right = Action::new();
        let mut move_left = Action::new();
        let mut w_scanner = Action::new();
        let mut w_down = Action::new();
        let mut w_up = Action::new();
        let mut jump = Action::new();
        let mut jump_w = Action::new();
        let mut first_mouse = Action::new();
        let mut second_mouse = Action::new();
        let mut mode_1 = Action::new();
        let mut mode_2 = Action::new();
        let mut mode_3 = Action::new();
        let mut activate_hand_slot_0 = Action::new();
        let mut activate_hand_slot_1 = Action::new();
        let mut activate_hand_slot_2 = Action::new();
        let mut activate_hand_slot_3 = Action::new();
        let mouse_axis = mouse_axis;
        
        for (_, (button_action, action)) in actions_table.iter() {
            match button_action {
                ButtonActions::MoveForward => move_forward = action.clone(),
                ButtonActions::MoveBackward => move_backward = action.clone(),
                ButtonActions::MoveRight => move_right = action.clone(),
                ButtonActions::MoveLeft => move_left = action.clone(),
                ButtonActions::WScaner => w_scanner = action.clone(),
                ButtonActions::Jump => jump = action.clone(),
                ButtonActions::JumpW => jump_w = action.clone(),
                ButtonActions::HandSlot0 => activate_hand_slot_0 = action.clone(),
                ButtonActions::HandSlot1=> activate_hand_slot_1 = action.clone(),
                ButtonActions::HandSlot2 => activate_hand_slot_2 = action.clone(),
                ButtonActions::HandSlot3 => activate_hand_slot_3 = action.clone(),
                ButtonActions::FirstMouse => first_mouse = action.clone(),
                ButtonActions::SecondMouse => second_mouse = action.clone(),
                ButtonActions::WDown => w_down = action.clone(),
                ButtonActions::WUp => w_up = action.clone(),
                ButtonActions::ModeOne => mode_1 = action.clone(),
                ButtonActions::ModeTwo => mode_2 = action.clone(),
                ButtonActions::ModeThree => mode_3 = action.clone(),
            }
        }

        ActionsFrameState {
            move_forward,
            move_backward,
            move_right,
            move_left,
            w_scanner,
            activate_hand_slot_0,
            activate_hand_slot_1,
            activate_hand_slot_2,
            activate_hand_slot_3,
            w_down,
            w_up,
            jump,
            jump_w,
            first_mouse,
            second_mouse,
            mode_1,
            mode_2,
            mode_3,
            mouse_axis
        }
    }

    pub fn empty() -> Self {
        let move_forward = Action::new();
        let move_backward = Action::new();
        let move_right = Action::new();
        let move_left = Action::new();
        let w_scanner = Action::new();
        let w_down = Action::new();
        let w_up = Action::new();
        let jump = Action::new();
        let jump_w = Action::new();
        let first_mouse = Action::new();
        let second_mouse = Action::new();
        let mode_1 = Action::new();
        let mode_2 = Action::new();
        let mode_3 = Action::new();
        let activate_hand_slot_0 = Action::new();
        let activate_hand_slot_1 = Action::new();
        let activate_hand_slot_2 = Action::new();
        let activate_hand_slot_3 = Action::new();
        let mouse_axis = Vec2::ZERO;

        ActionsFrameState {
            move_forward,
            move_backward,
            move_right,
            move_left,
            w_scanner,
            activate_hand_slot_0,
            activate_hand_slot_1,
            activate_hand_slot_2,
            activate_hand_slot_3,
            w_down,
            w_up,
            jump,
            jump_w,
            first_mouse,
            second_mouse,
            mode_1,
            mode_2,
            mode_3,
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
    HandSlot0,
    HandSlot1,
    HandSlot2,
    HandSlot3,
    WScaner,
    JumpW,
    Jump,
    WUp,
    WDown,
    ModeOne,
    ModeTwo,
    ModeThree,
    FirstMouse,
    SecondMouse,
}

// for future user's settings
enum ButtonActionsLinkedKeys {
    MoveForward(KeyCode),
    MoveBackward(KeyCode),
    MoveRight(KeyCode),
    MoveLeft(KeyCode),
    Jump(KeyCode),
    FirstMouse(KeyCode),
    SecondMouse(KeyCode),
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
            SomeButton::KeyCode(KeyCode::ShiftLeft),
            (ButtonActions::JumpW, Action::new())
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::Space),
            (ButtonActions::Jump, Action::new())
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyE),
            (ButtonActions::WScaner, Action::new())
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyQ),
            (ButtonActions::WDown, Action::new())
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::Digit1),
            (ButtonActions::HandSlot0, Action::new())
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::Digit2),
            (ButtonActions::HandSlot1, Action::new())
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::Digit3),
            (ButtonActions::HandSlot2, Action::new())
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::Digit4),
            (ButtonActions::HandSlot3, Action::new())
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::Numpad1),
            (ButtonActions::ModeOne, Action::new())
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::Numpad2),
            (ButtonActions::ModeTwo, Action::new())
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::Numpad3),
            (ButtonActions::ModeThree, Action::new())
        );
        actions_table.insert(
            SomeButton::MouseButton(MouseButton::Left),
            (ButtonActions::FirstMouse, Action::new())
        );
        actions_table.insert(
            SomeButton::MouseButton(MouseButton::Right),
            (ButtonActions::SecondMouse, Action::new())
        );
        InputSystem {
            actions_table,
            mouse_axis: Vec2::ZERO,
        }
    }

    pub fn get_input(&mut self, world: &mut World ,net: &mut NetSystem) {

        for (_, actor) in world.actors.iter_mut() {

            if let ActorWrapper::Player(player) = actor {
                
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
    }




    pub fn reset_input(&mut self) {

        for (_, (_, action)) in self.actions_table.iter_mut() {
            action.is_action_just_pressed = false;
        }

        self.mouse_axis = Vec2::ZERO;
    }

    pub fn add_mouse_delta(&mut self, delta: Vec2) {
        self.mouse_axis += delta * -MOUSE_SENSITIVITY;
    }

    pub fn set_keyboard_input(&mut self, input: &KeyEvent) {

        if let PhysicalKey::Code(keycode) = input.physical_key {
    
            if let Some((_, action)) =
                self.actions_table.get_mut(&SomeButton::KeyCode(keycode)) {

                match input.state {
                    ElementState::Pressed => {
                        if action.is_action_pressed {
                            action.is_action_just_pressed = false;
                        } else {
                            action.is_action_just_pressed = true;
                        }
                        
                        action.is_action_pressed = true;
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