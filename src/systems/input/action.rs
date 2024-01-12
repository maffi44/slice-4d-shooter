#[derive(Clone, Debug)]
pub struct Action {
    pub is_action_just_pressed: bool,
    pub is_action_pressed: bool,
}

impl Action {
    pub fn new() -> Self {
        Action {
            is_action_pressed: false,
            is_action_just_pressed: false,
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