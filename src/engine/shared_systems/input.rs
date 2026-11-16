// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



pub mod action;

use self::action::Action;

use std::collections::HashMap;

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
    #[allow(dead_code)] // input API (W actions, gamepad sticks), not used by the game yet
    pub w_down: Action,
    #[allow(dead_code)] // input API (W actions, gamepad sticks), not used by the game yet
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
        actions: &[(ButtonActions, Action, i32)],
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
        
        for action_pair in actions.iter() {
            
            let (action_type, action, _) = action_pair;

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

// for future user settings


pub struct InputSystem {
    /// All input actions with their state and pressed-buttons counter.
    actions: Vec<(ButtonActions, Action, i32)>,
    /// Button → index into `actions`. Several buttons (keyboard, mouse, gamepad) can share one action.
    actions_table: HashMap<SomeButton, usize>,
    /// The actions of the current tick, computed once in `update_frame_actions` (pre-simulate).
    frame_actions: ActionsFrameState,
    mouse_axis: Vec2,
    gamepad_left_stick_axis: Vec2,
    gamepad_right_stick_axis: Vec2,
    mouse_wheel_delta: Vec2,
    gilrs: Gilrs,
}

impl InputSystem {

    pub fn new() -> Self {
        let mut actions: Vec<(ButtonActions, Action, i32)> = Vec::new();
        let mut actions_table = HashMap::new();

        let move_forward = add_action(&mut actions, ButtonActions::MoveForward);
        let move_backward = add_action(&mut actions, ButtonActions::MoveBackward);
        let move_right = add_action(&mut actions, ButtonActions::MoveRight);
        let move_left = add_action(&mut actions, ButtonActions::MoveLeft);
        let jump_w = add_action(&mut actions, ButtonActions::JumpW);
        let jump = add_action(&mut actions, ButtonActions::Jump);
        let w_scanner = add_action(&mut actions, ButtonActions::WScanner);
        let anti_projection_mode = add_action(&mut actions, ButtonActions::AntiProjectionMode);
        let first_mouse = add_action(&mut actions, ButtonActions::FirstMouse);
        let second_mouse = add_action(&mut actions, ButtonActions::SecondMouse);
        let middle_mouse = add_action(&mut actions, ButtonActions::MiddleMouse);
        let hand_slot_0 = add_action(&mut actions, ButtonActions::HandSlot0);
        let hand_slot_1 = add_action(&mut actions, ButtonActions::HandSlot1);
        let enable_w_aim = add_action(&mut actions, ButtonActions::EnableWAim);
        let show_hide_controls = add_action(&mut actions, ButtonActions::ShowHideControls);

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
            move_left.clone(),
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
            move_left.clone(),
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
            SomeButton::GamepadButton(Button::East),
            enable_w_aim.clone()
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
            SomeButton::GamepadButton(Button::Start),
            show_hide_controls.clone()
        );

        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyJ),
            add_action(&mut actions, ButtonActions::WUp)
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyK),
            add_action(&mut actions, ButtonActions::WDown)
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::Digit3),
            add_action(&mut actions, ButtonActions::HandSlot2)
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::Digit4),
            add_action(&mut actions, ButtonActions::HandSlot3)
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyT),
            show_hide_controls,
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyR),
            enable_w_aim
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyI),
            add_action(&mut actions, ButtonActions::DecreaseRenderQuality)
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyO),
            add_action(&mut actions, ButtonActions::IncreaseRenderQuality)
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyP),
            add_action(&mut actions, ButtonActions::ShadowsToggle)
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::KeyG),
            add_action(&mut actions, ButtonActions::ConnectToServer)
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::ArrowUp),
            add_action(&mut actions, ButtonActions::ArrowUp)
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::ArrowDown),
            add_action(&mut actions, ButtonActions::ArrowDown)
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::ArrowLeft),
            add_action(&mut actions, ButtonActions::ArrowLeft)
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::ArrowRight),
            add_action(&mut actions, ButtonActions::ArrowRight)
        );
        actions_table.insert(
            SomeButton::KeyCode(KeyCode::ControlRight),
            add_action(&mut actions, ButtonActions::MoveCameraBackInExample)
        );
        

        InputSystem {
            actions,
            actions_table,
            frame_actions: ActionsFrameState::empty(),
            mouse_axis: Vec2::ZERO,
            gamepad_right_stick_axis: Vec2::ZERO,
            gamepad_left_stick_axis: Vec2::ZERO,
            mouse_wheel_delta: Vec2::ZERO,
            gilrs: Gilrs::new().unwrap(),
        }
    }


    /// Computes the actions of the current tick (called once per engine tick, before simulation).
    pub fn update_frame_actions(&mut self)
    {
        self.frame_actions = self.get_input();
    }

    /// The input actions of the current tick. Any actor of any world reads them via `shared.input`.
    #[inline]
    pub fn actions(&self) -> &ActionsFrameState
    {
        &self.frame_actions
    }


    pub fn get_input(&self) -> ActionsFrameState
    {
        ActionsFrameState::current(
            &self.actions,
            self.mouse_axis,
            self.mouse_wheel_delta,
            self.gamepad_right_stick_axis,
            self.gamepad_left_stick_axis,
        )
    }


    pub fn reset_input(&mut self) {

        for action_pair in self.actions.iter_mut() {
            action_pair.1.is_action_just_pressed = false;
        }

        self.mouse_axis = Vec2::ZERO;
        // self.gamepad_right_stick_axis = Vec2::ZERO;
        // self.gamepad_left_stick_axis = Vec2::ZERO;
        self.mouse_wheel_delta = Vec2::ZERO;
    }

    pub fn add_mouse_delta(&mut self, delta: Vec2) {
        self.mouse_axis += delta * -MOUSE_SENSITIVITY;
    }

    #[allow(dead_code)] // input API (W actions, gamepad sticks), not used by the game yet
    pub fn add_gamepad_right_stick_axis_delta(&mut self, delta: Vec2) {
        self.gamepad_right_stick_axis += delta * -MOUSE_SENSITIVITY;
    }

    #[allow(dead_code)] // input API (W actions, gamepad sticks), not used by the game yet
    pub fn add_gamepad_left_stick_axis_delta(&mut self, delta: Vec2) {
        self.gamepad_left_stick_axis += delta * -MOUSE_SENSITIVITY;
    }

    pub fn set_keyboard_input(&mut self, input: &KeyEvent) {

        if let PhysicalKey::Code(keycode) = input.physical_key {
    
            if let Some(action_pair) =
            self.actions_table.get(&SomeButton::KeyCode(keycode)) {

                let (_,action,pressed_count) = &mut self.actions[*action_pair];

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
        self.actions_table.get(&SomeButton::GamepadButton(button)) {

            let (_,action,pressed_count) = &mut self.actions[*action_pair];

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

    #[allow(dead_code)] // input API (W actions, gamepad sticks), not used by the game yet
    pub fn set_keyboard_input_by_keycode(&mut self, keycode: KeyCode, state: ElementState) {
    
        if let Some(action_pair) =
        self.actions_table.get(&SomeButton::KeyCode(keycode)) {

            let (_,action,pressed_count) = &mut self.actions[*action_pair];

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
                self.actions_table.get(&SomeButton::MouseButton(MouseButton::Left)) {

                    let (_,action,pressed_count) = &mut self.actions[*action_pair];

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
                self.actions_table.get(&SomeButton::MouseButton(MouseButton::Middle)) {
                    
                    let (_,action,pressed_count) = &mut self.actions[*action_pair];
                    
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
                self.actions_table.get(&SomeButton::MouseButton(MouseButton::Right)) {
                    
                    let (_,action,pressed_count) = &mut self.actions[*action_pair];

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
                self.actions_table.get(&SomeButton::MouseButton(MouseButton::Other(*code))) {

                    let (_,action,pressed_count) = &mut self.actions[*action_pair];

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
                gilrs::EventType::ButtonPressed(button, _code) =>
                {
                    self.set_gamepad_input(button, ElementState::Pressed);
                }
                gilrs::EventType::ButtonReleased(button, _code) =>
                {
                    self.set_gamepad_input(button, ElementState::Released);

                },
                gilrs::EventType::AxisChanged(axis, delta, _code) => {
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

fn add_action(actions: &mut Vec<(ButtonActions, Action, i32)>, action_type: ButtonActions) -> usize
{
    actions.push((action_type, Action::new(), 0));

    actions.len() - 1
}
