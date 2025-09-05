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

use crate::engine::input::ActionsFrameState;

pub enum InputMaster {
    LocalMaster(LocalMaster),
    RemoteMaster(RemoteMaster),
}

pub struct LocalMaster {
    pub current_input: ActionsFrameState
}

impl LocalMaster {
    pub fn new(init_atctions: ActionsFrameState) -> Self {

        LocalMaster {
            current_input: init_atctions,
        }
    }
}


pub struct RemoteMaster {
    pub current_input: ActionsFrameState
}


// pub struct WrappedVec<T> {
//     buf: Vec<T>,
    
// }