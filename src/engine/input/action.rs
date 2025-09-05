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

#[derive(Clone, Debug)]
pub struct Action {
    pub(super) is_action_just_pressed: bool,
    pub(super) is_action_pressed: bool,
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