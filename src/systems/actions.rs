pub static mut MOVE_FORWARD: Action = Action::new();
pub static mut MOVE_BACKWARD: Action = Action::new();
pub static mut MOVE_RIGHT: Action = Action::new();
pub static mut MOVE_LEFT: Action = Action::new();
pub static mut JUMP: Action = Action::new();
pub static mut FIRE: Action = Action::new();

use glam::Vec2;

#[derive(Clone)]
pub struct Action {
    pub is_action_just_pressed: bool,
    pub is_action_pressed: bool,
    // I'm don't sure that this field is really necessary
    // pub already_captured: bool,
}


#[derive(Clone)]
pub struct Actions {
    pub move_forward: Action,
    pub move_backward: Action,
    pub move_right: Action,
    pub move_left: Action,
    pub jump: Action,
    pub fire: Action,
    pub axis_input: Vec2,
}

impl Actions {
    pub fn new() -> Self {
        unsafe {
            let move_forward = MOVE_FORWARD.clone();
            let move_backward = MOVE_FORWARD.clone();
            let move_right = MOVE_FORWARD.clone();
            let move_left = MOVE_FORWARD.clone();
            let jump = MOVE_FORWARD.clone();
            let fire = MOVE_FORWARD.clone();
            
            Actions {
                move_forward,
                move_backward,
                move_right,
                move_left,
                jump,
                fire,
                axis_input: Vec2::ZERO,
            }
        }
    }
}

impl Action {
    const fn new() -> Self {
        Action {
            is_action_pressed: false,
            is_action_just_pressed: false,
            // already_captured: false,
        }
    }

    pub fn is_action_just_pressed(&self) -> bool {
        self.is_action_just_pressed
    }

    pub fn is_action_pressed(&self) -> bool {
        self.is_action_pressed
    }

    pub fn capture_action(&mut self) {
        self.is_action_just_pressed = false;
        self.is_action_pressed = false;
    }
}