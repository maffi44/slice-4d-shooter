// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2025  Timofei Molokov

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

pub mod action;

use crate::{
    actor::{
        main_player::player_input_master::InputMaster::{
            LocalMaster,
            RemoteMaster,
        }, Actor
    }, engine::{
        net::NetSystem,
        world::World,
    }
};

use self::action::Action;

use std::{collections::HashMap, sync::{Arc, Mutex}};

use gilrs::{Button, Event, Gilrs};
use winit::{
    event::{
        ElementState, KeyEvent, MouseButton, MouseScrollDelta
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
    pub anti_projection_mode: Action,
    pub activate_hand_slot_0: Action,
    pub activate_hand_slot_1: Action,
    pub activate_hand_slot_2: Action,
    pub activate_hand_slot_3: Action,
    pub show_hide_controls: Action,
    pub jump: Action,
    pub jump_w: Action,
    pub first_mouse: Action,
    pub second_mouse: Action,
    pub middle_mouse: Action,
    pub w_aim: Action,
    pub increase_render_quality: Action,
    pub decrease_render_quality: Action,
    pub shadows_toggle: Action,
    pub connect_to_server: Action,
    pub arrow_up: Action,
    pub arrow_down: Action,
    pub arrow_left: Action,
    pub arrow_right: Action,
    pub move_camera_back_in_example: Action,
    pub mouse_axis: Vec2,
    pub mouse_wheel_delta: Vec2,
    pub gamepad_right_stick_axis_delta: Vec2,
    pub gamepad_left_stick_axis_delta: Vec2,
    
}

impl ActionsFrameState {
    fn current(
        actions_table: &HashMap<SomeButton, Arc<Mutex<(ButtonActions, Action, i32)>>>,
        mouse_axis: Vec2,
        mouse_wheel_delta: Vec2,
        gamepad_right_stick_axis: Vec2,
        gamepad_left_stick_axis: Vec2,
    ) -> Self {
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
        let mut middle_mouse = Action::new();
        let mut activate_hand_slot_0 = Action::new();
        let mut activate_hand_slot_1 = Action::new();
        let mut activate_hand_slot_2 = Action::new();
        let mut activate_hand_slot_3 = Action::new();
        let mut enable_w_aim = Action::new();
        let mut show_hide_controls = Action::new();
        let mut increase_render_quality = Action:: new();
        let mut decrease_render_quality = Action:: new();
        let mut shadows_toggle = Action:: new();
        let mut connect_to_server = Action:: new();
        let mut arrow_up = Action::new();
        let mut arrow_down = Action::new();
        let mut arrow_left = Action::new();
        let mut arrow_right = Action::new();
        let mut move_camera_back_in_example = Action::new();
        let mut anti_projection_mode = Action::new();
        let mouse_axis = mouse_axis;
        
        for (_, action_pair) in actions_table.iter() {
            
            let (action_type, action, _) = &mut *action_pair.lock().unwrap();

            match action_type {
                ButtonActions::MoveForward => move_forward = action.clone(),
                ButtonActions::MoveBackward => move_backward = action.clone(),
                ButtonActions::MoveRight => move_right = action.clone(),
                ButtonActions::MoveLeft => move_left = action.clone(),
                ButtonActions::WScanner => w_scanner = action.clone(),
                ButtonActions::Jump => jump = action.clone(),
                ButtonActions::JumpW => jump_w = action.clone(),
                ButtonActions::HandSlot0 => activate_hand_slot_0 = action.clone(),
                ButtonActions::HandSlot1=> activate_hand_slot_1 = action.clone(),
                ButtonActions::HandSlot2 => activate_hand_slot_2 = action.clone(),
                ButtonActions::HandSlot3 => activate_hand_slot_3 = action.clone(),
                ButtonActions::FirstMouse => first_mouse = action.clone(),
                ButtonActions::SecondMouse => second_mouse = action.clone(),
                ButtonActions::MiddleMouse => middle_mouse = action.clone(),
                ButtonActions::WDown => w_down = action.clone(),
                ButtonActions::WUp => w_up = action.clone(),
                ButtonActions::EnableWAim => enable_w_aim = action.clone(),
                ButtonActions::ShowHideControls => show_hide_controls = action.clone(),
                ButtonActions::IncreaseRenderQuality => increase_render_quality = action.clone(),
                ButtonActions::DecreaseRenderQuality => decrease_render_quality = action.clone(),
                ButtonActions::ShadowsToggle => shadows_toggle = action.clone(),
                ButtonActions::ConnectToServer => connect_to_server = action.clone(),
                ButtonActions::ArrowUp => arrow_up = action.clone(),
                ButtonActions::ArrowDown => arrow_down = action.clone(),
                ButtonActions::ArrowLeft => arrow_left = action.clone(),
                ButtonActions::ArrowRight => arrow_right = action.clone(),
                ButtonActions::MoveCameraBackInExample => move_camera_back_in_example = action.clone(),
                ButtonActions::AntiProjectionMode => anti_projection_mode = action.clone(),
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
            middle_mouse,
            mouse_axis,
            w_aim: enable_w_aim,
            increase_render_quality,
            decrease_render_quality,
            shadows_toggle,
            show_hide_controls,
            connect_to_server,
            arrow_up,
            arrow_down,
            arrow_left,
            arrow_right,
            move_camera_back_in_example,
            anti_projection_mode,
            mouse_wheel_delta,
            gamepad_left_stick_axis_delta: gamepad_left_stick_axis,
            gamepad_right_stick_axis_delta: gamepad_right_stick_axis,
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
        let middle_mouse = Action::new();
        let activate_hand_slot_0 = Action::new();
        let activate_hand_slot_1 = Action::new();
        let activate_hand_slot_2 = Action::new();
        let activate_hand_slot_3 = Action::new();
        let w_aim = Action::new();
        let show_hide_controls = Action::new();
        let increase_render_quality = Action:: new();
        let decrease_render_quality = Action:: new();
        let shadows_toggle = Action:: new();
        let connect_to_server = Action::new();
        let arrow_up = Action::new();
        let arrow_down = Action::new();
        let arrow_left = Action::new();
        let arrow_right = Action::new();
        let move_camera_back_in_example = Action::new();
        let anti_projection_mode = Action::new();
        let mouse_axis = Vec2::ZERO;
        let gamepad_left_stick_axis_delta = Vec2::ZERO;
        let gamepad_right_stick_axis_delta = Vec2::ZERO;
        let mouse_wheel_delta = Vec2::ZERO;

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
            show_hide_controls,
            first_mouse,
            second_mouse,
            middle_mouse,
            w_aim,
            increase_render_quality,
            decrease_render_quality,
            shadows_toggle,
            connect_to_server,
            arrow_up,
            arrow_down,
            arrow_left,
            arrow_right,
            move_camera_back_in_example,
            anti_projection_mode,
            mouse_axis,
            gamepad_left_stick_axis_delta,
            gamepad_right_stick_axis_delta,
            mouse_wheel_delta,
        }
    }
}

pub enum MouseAxis {
    X,
    Y
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum SomeButton {
    MouseButton(MouseButton),
    KeyCode(KeyCode),
    GamepadButton(Button)
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum ButtonActions {
    MoveForward,
    MoveBackward,
    MoveRight,
    MoveLeft,
    HandSlot0,
    HandSlot1,
    HandSlot2,
    HandSlot3,
    WScanner,
    AntiProjectionMode,
    JumpW,
    ShowHideControls,
    Jump,
    WUp,
    WDown,
    FirstMouse,
    SecondMouse,
    MiddleMouse,
    EnableWAim,
    IncreaseRenderQuality,
    DecreaseRenderQuality,
    ConnectToServer,
    ShadowsToggle,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    MoveCameraBackInExample,
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
    actions_table: HashMap<SomeButton, Arc<Mutex<(ButtonActions, Action, i32)>>>,
    mouse_axis: Vec2,
    gamepad_left_stick_axis: Vec2,
    gamepad_right_stick_axis: Vec2,
    mouse_wheel_delta: Vec2,
    gilrs: Gilrs,
}

impl InputSystem {

    pub fn new() -> Self {
        let mut actions_table = HashMap::new();

        let move_forward = Arc::new(Mutex::new((ButtonActions::MoveForward, Action::new(), 0)));
        let move_backward = Arc::new(Mutex::new((ButtonActions::MoveBackward, Action::new(), 0)));
        let move_right = Arc::new(Mutex::new((ButtonActions::MoveRight, Action::new(), 0)));
        let move_letf = Arc::new(Mutex::new((ButtonActions::MoveLeft, Action::new(), 0)));
        let jump_w = Arc::new(Mutex::new((ButtonActions::JumpW, Action::new(), 0)));
        let jump = Arc::new(Mutex::new((ButtonActions::Jump, Action::new(), 0)));
        let w_scanner = Arc::new(Mutex::new((ButtonActions::WScanner, Action::new(), 0)));
        let anti_projection_mode = Arc::new(Mutex::new((ButtonActions::AntiProjectionMode, Action::new(), 0)));
        let first_mouse = Arc::new(Mutex::new((ButtonActions::FirstMouse, Action::new(), 0)));
        let second_mouse = Arc::new(Mutex::new((ButtonActions::SecondMouse, Action::new(), 0)));
        let middle_mouse = Arc::new(Mutex::new((ButtonActions::MiddleMouse, Action::new(), 0)));
        let hand_slot_0 = Arc::new(Mutex::new((ButtonActions::HandSlot0, Action::new(), 0)));
        let hand_slot_1 = Arc::new(Mutex::new((ButtonActions::HandSlot1, Action::new(), 0)));

        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyW),
            move_forward.clone(),
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyS),
            move_backward.clone(),
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyD),
            move_right.clone(),
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyA),
            move_letf.clone(),
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::ShiftLeft),
            jump_w.clone(),
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::Space),
            jump.clone(),
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyE),
            w_scanner.clone(),
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyQ),
            anti_projection_mode.clone(),
        );
        actions_table.insert(
            SomeButton::MouseButton(MouseButton::Left),
            first_mouse.clone(),
        );
        actions_table.insert(
            SomeButton::MouseButton(MouseButton::Right),
            second_mouse.clone(),
        );
        actions_table.insert(
            SomeButton::MouseButton(MouseButton::Middle),
            middle_mouse.clone(),
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::Digit1),
            hand_slot_0.clone(),
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::Digit2),
            hand_slot_1.clone(),
        );

        actions_table.insert(
            SomeButton::GamepadButton(Button::DPadUp),
            move_forward.clone(),
        );
        actions_table.insert(
            SomeButton::GamepadButton(Button::DPadDown),
            move_backward.clone(),
        );
        actions_table.insert(
            SomeButton::GamepadButton(Button::DPadRight),
            move_right.clone(),
        );
        actions_table.insert(
            SomeButton::GamepadButton(Button::DPadLeft),
            move_letf.clone(),
        );
        actions_table.insert(
            SomeButton::GamepadButton(Button::East),
            jump_w.clone(),
        );
        actions_table.insert(
            SomeButton::GamepadButton(Button::South),
            jump.clone(),
        );
        actions_table.insert(
            SomeButton::GamepadButton(Button::RightThumb),
            jump.clone(),
        );
        actions_table.insert(
            SomeButton::GamepadButton(Button::North),
            w_scanner.clone(),
        );
        actions_table.insert(
            SomeButton::GamepadButton(Button::West),
            anti_projection_mode.clone(),
        );
        actions_table.insert(
            SomeButton::GamepadButton(Button::RightTrigger),
            first_mouse.clone(),
        );
        actions_table.insert(
            SomeButton::GamepadButton(Button::RightTrigger2),
            second_mouse.clone(),
        );
        actions_table.insert(
            SomeButton::GamepadButton(Button::LeftTrigger),
            hand_slot_0
        );
        actions_table.insert(
            SomeButton::GamepadButton(Button::LeftTrigger2),
            hand_slot_1
        );

        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyJ),
            Arc::new(Mutex::new((ButtonActions::WUp, Action::new(), 0)))
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyK),
            Arc::new(Mutex::new((ButtonActions::WDown, Action::new(), 0)))
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::Digit3),
            Arc::new(Mutex::new((ButtonActions::HandSlot2, Action::new(), 0)))
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::Digit4),
            Arc::new(Mutex::new((ButtonActions::HandSlot3, Action::new(), 0)))
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyT),
            Arc::new(Mutex::new((ButtonActions::ShowHideControls, Action::new(), 0)))
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyR),
            Arc::new(Mutex::new((ButtonActions::EnableWAim, Action::new(), 0)))
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyI),
            Arc::new(Mutex::new((ButtonActions::DecreaseRenderQuality, Action::new(), 0)))
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyO),
            Arc::new(Mutex::new((ButtonActions::IncreaseRenderQuality, Action::new(), 0)))
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyP),
            Arc::new(Mutex::new((ButtonActions::ShadowsToggle, Action::new(), 0)))
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyG),
            Arc::new(Mutex::new((ButtonActions::ConnectToServer, Action::new(), 0)))
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::ArrowUp),
            Arc::new(Mutex::new((ButtonActions::ArrowUp, Action::new(), 0)))
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::ArrowDown),
            Arc::new(Mutex::new((ButtonActions::ArrowDown, Action::new(), 0)))
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::ArrowLeft),
            Arc::new(Mutex::new((ButtonActions::ArrowLeft, Action::new(), 0)))
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::ArrowRight),
            Arc::new(Mutex::new((ButtonActions::ArrowRight, Action::new(), 0)))
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::ControlRight),
            Arc::new(Mutex::new((ButtonActions::MoveCameraBackInExample, Action::new(), 0)))
        );
        

        InputSystem {
            actions_table,
            mouse_axis: Vec2::ZERO,
            gamepad_right_stick_axis: Vec2::ZERO,
            gamepad_left_stick_axis: Vec2::ZERO,
            mouse_wheel_delta: Vec2::ZERO,
            gilrs: Gilrs::new().unwrap(),
        }
    }


    pub fn get_actions_table(&self) -> &HashMap<SomeButton, Arc<Mutex<(ButtonActions, Action, i32)>>>
    {
        &self.actions_table
    }


    pub fn set_input_to_controlled_actors(&self, world: &mut World ,net: &mut NetSystem) {

        for (_, actor) in world.actors.iter_mut() {

            if let Some(controlled_actor) = actor.get_actor_as_controlled_mut()
            {
                match controlled_actor.get_input_master() {
                    LocalMaster(master) => {
                        
                        master.current_input =
                            ActionsFrameState::current(
                                &self.actions_table,
                                self.mouse_axis,
                                self.mouse_wheel_delta,
                                self.gamepad_right_stick_axis,
                                self.gamepad_left_stick_axis,
                            );
                    }
                    RemoteMaster(master) => {
                        unimplemented!("Remote input aster didn't implement yet")
                    }
                }
            }
        }
    }


    pub fn get_input(&self) -> ActionsFrameState
    {
        ActionsFrameState::current(
            &self.actions_table,
            self.mouse_axis,
            self.mouse_wheel_delta,
            self.gamepad_right_stick_axis,
            self.gamepad_left_stick_axis,
        )
    }


    pub fn reset_input(&mut self) {

        for (_, action_pair) in self.actions_table.iter_mut() {
            action_pair.lock().unwrap().1.is_action_just_pressed = false;
        }

        self.mouse_axis = Vec2::ZERO;
        // self.gamepad_right_stick_axis = Vec2::ZERO;
        // self.gamepad_left_stick_axis = Vec2::ZERO;
        self.mouse_wheel_delta = Vec2::ZERO;
    }

    pub fn add_mouse_delta(&mut self, delta: Vec2) {
        self.mouse_axis += delta * -MOUSE_SENSITIVITY;
    }

    pub fn add_gamepad_right_stick_axis_delta(&mut self, delta: Vec2) {
        self.gamepad_right_stick_axis += delta * -MOUSE_SENSITIVITY;
    }

    pub fn add_gamepad_left_stick_axis_delta(&mut self, delta: Vec2) {
        self.gamepad_left_stick_axis += delta * -MOUSE_SENSITIVITY;
    }

    pub fn set_keyboard_input(&mut self, input: &KeyEvent) {

        if let PhysicalKey::Code(keycode) = input.physical_key {
    
            if let Some(action_pair) =
            self.actions_table.get_mut(&SomeButton::KeyCode(keycode)) {

                let (_,action,pressed_count) = &mut *action_pair.lock().unwrap();

                match input.state {
                    ElementState::Pressed => {
                        *pressed_count += 1;

                        if action.is_action_pressed {
                            action.is_action_just_pressed = false;
                        } else {
                            action.is_action_just_pressed = true;
                        }
                        
                        action.is_action_pressed = true;
                    },
                    ElementState::Released => {
                        *pressed_count -= 1;

                        action.is_action_just_pressed = false;
                        action.is_action_pressed = false;
                        if *pressed_count <= 0
                        {
                            *pressed_count = 0;
                        }
                    }
                }
            }
        }
    }

    pub fn set_gamepad_input(&mut self, button: Button, state: ElementState) {
    
        if let Some(action_pair) =
        self.actions_table.get_mut(&SomeButton::GamepadButton(button)) {

            let (_,action,pressed_count) = &mut *action_pair.lock().unwrap();

            match state {
                ElementState::Pressed => {
                    *pressed_count += 1;

                    if action.is_action_pressed {
                        action.is_action_just_pressed = false;
                    } else {
                        action.is_action_just_pressed = true;
                    }
                    
                    action.is_action_pressed = true;
                },
                ElementState::Released => {
                    *pressed_count -= 1;

                    action.is_action_just_pressed = false;
                    action.is_action_pressed = false;
                    if *pressed_count <= 0
                    {
                        *pressed_count = 0;
                    }
                }
             }
        }
    }

    pub fn set_keyboard_input_by_keycode(&mut self, keycode: KeyCode, state: ElementState) {
    
        if let Some(action_pair) =
        self.actions_table.get_mut(&SomeButton::KeyCode(keycode)) {

            let (_,action,pressed_count) = &mut *action_pair.lock().unwrap();

            match state {
                ElementState::Pressed => {
                    *pressed_count += 1;

                    if action.is_action_pressed {
                        action.is_action_just_pressed = false;
                    } else {
                        action.is_action_just_pressed = true;
                    }
                    
                    action.is_action_pressed = true;
                },
                ElementState::Released => {
                    *pressed_count -= 1;

                    action.is_action_just_pressed = false;
                    action.is_action_pressed = false;
                    if *pressed_count <= 0
                    {
                        *pressed_count = 0;
                    }
                }
            }
        }
    }

    pub fn set_mouse_wheel_input(&mut self, delta: MouseScrollDelta) {

        match delta
        {
            MouseScrollDelta::LineDelta(x, y) => {
                self.mouse_wheel_delta = Vec2::new(x,y);
            },
            MouseScrollDelta::PixelDelta(pos) => {
                self.mouse_wheel_delta = Vec2::new(pos.x as f32, pos.y as f32);
            },
        }
    }

    pub fn set_mouse_button_input(&mut self, button: &MouseButton, state: &ElementState) {
        match button {
            MouseButton::Left => {
                if let Some(action_pair) =
                self.actions_table.get_mut(&SomeButton::MouseButton(MouseButton::Left)) {

                    let (_,action,pressed_count) = &mut *action_pair.lock().unwrap();

                    match state {
                        ElementState::Pressed => {
                            *pressed_count += 1;

                            if action.is_action_pressed == false {
                                action.is_action_just_pressed = true;
                                action.is_action_pressed = true;

                            } else {
                                action.is_action_just_pressed = false;
                                action.is_action_pressed = true;
                            }
                        },
                        ElementState::Released => {
                            *pressed_count -= 1;

                            action.is_action_just_pressed = false;
                            action.is_action_pressed = false;
                            if *pressed_count <= 0
                            {
                                *pressed_count = 0;
                            }
                        },
                    }

                    // action.already_captured = false;
                }
            },
            MouseButton::Middle => {
                if let Some(action_pair) =
                self.actions_table.get_mut(&SomeButton::MouseButton(MouseButton::Middle)) {
                    
                    let (_,action,pressed_count) = &mut *action_pair.lock().unwrap();
                    
                    match state {
                        ElementState::Pressed => {
                            *pressed_count += 1;

                            if action.is_action_pressed == false {
                                action.is_action_just_pressed = true;
                                action.is_action_pressed = true;
                            } else {
                                action.is_action_just_pressed = false;
                                action.is_action_pressed = true;
                            }
                        },
                        ElementState::Released => {
                            *pressed_count -= 1;

                            action.is_action_just_pressed = false;
                            action.is_action_pressed = false;
                            if *pressed_count <= 0
                            {
                                *pressed_count = 0;
                            }
                        },
                    }
                    // action.already_captured = false;
                }
            },
            MouseButton::Right => {
                if let Some(action_pair) =
                self.actions_table.get_mut(&SomeButton::MouseButton(MouseButton::Right)) {
                    
                    let (_,action,pressed_count) = &mut *action_pair.lock().unwrap();

                    match state {
                        ElementState::Pressed => {
                            *pressed_count += 1;

                            if action.is_action_pressed == false {
                                action.is_action_just_pressed = true;
                                action.is_action_pressed = true;
                            } else {
                                action.is_action_just_pressed = false;
                                action.is_action_pressed = true;
                            }
                        },
                        ElementState::Released => {
                            *pressed_count -= 1;

                            action.is_action_just_pressed = false;
                            action.is_action_pressed = false;
                            if *pressed_count <= 0
                            {
                                *pressed_count = 0;
                            }
                        },
                    }
                    // action.already_captured = false;
                }
            },
            MouseButton::Other(code) => {
                if let Some(action_pair) =
                self.actions_table.get_mut(&SomeButton::MouseButton(MouseButton::Other(*code))) {

                    let (_,action,pressed_count) = &mut *action_pair.lock().unwrap();

                    match state {
                        ElementState::Pressed => {
                            *pressed_count += 1;

                            if action.is_action_pressed == false {
                                action.is_action_just_pressed = true;
                                action.is_action_pressed = true;
                            } else {
                                action.is_action_just_pressed = false;
                                action.is_action_pressed = true;
                            }
                        },
                        ElementState::Released => {
                            *pressed_count -= 1;

                            action.is_action_just_pressed = false;
                            action.is_action_pressed = false;
                            if *pressed_count <= 0
                            {
                                *pressed_count = 0;
                            }
                        },
                    }
                    // action.already_captured = false;
                }
            },
            MouseButton::Back => {},
            MouseButton::Forward => {},
        }
    }

    #[inline]
    pub fn collect_gamepad_button_input(&mut self)
    {
        while let Some(Event {event, ..}) = self.gilrs.next_event()
        {
            match event
            {
                gilrs::EventType::ButtonPressed(button, code) =>
                {
                    self.set_gamepad_input(button, ElementState::Pressed);
                }
                gilrs::EventType::ButtonReleased(button, code) =>
                {
                    self.set_gamepad_input(button, ElementState::Released);

                },
                gilrs::EventType::AxisChanged(axis, delta, code) => {
                    match axis
                    {
                        gilrs::Axis::LeftStickX =>
                        {
                            self.gamepad_left_stick_axis.x = delta;
                        },
                        gilrs::Axis::LeftStickY =>
                        {
                            self.gamepad_left_stick_axis.y = delta;
                        },
                        gilrs::Axis::RightStickX =>
                        {
                            self.gamepad_right_stick_axis.x = delta;
                        },
                        gilrs::Axis::RightStickY =>
                        {
                            self.gamepad_right_stick_axis.y = delta;
                        },
                        _ => {}
                    }
                },
                gilrs::EventType::Disconnected => {

                },
                _ => {},
            }
        }
    }
}