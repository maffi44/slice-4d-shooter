#[derive(Clone, Debug)]
pub struct Action {
    pub is_action_just_pressed: bool,
    pub is_action_pressed: bool,
    pub already_set_in_current_frame: bool,
}

impl Action {
    pub fn new() -> Self {
        Action {
            is_action_pressed: false,
            is_action_just_pressed: false,
            already_set_in_current_frame: false,
        }
    }

    pub fn is_action_just_pressed(&self) -> bool {
        self.is_action_just_pressed
    }

    pub fn is_action_pressed(&self) -> bool {
        self.is_action_pressed
    }
}