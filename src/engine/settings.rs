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

use crate::actor::main_player::player_settings::PlayerSettings;

pub struct Settings
{
    player_settings: PlayerSettings,
}

impl Settings
{
    pub fn new(
        player_settings: PlayerSettings
    ) -> Self
    {
        Settings {
            player_settings,
        }
    }

    pub fn increase_mouse_sensitivity(&self, delta: f32)
    {
        let mut s = self
            .player_settings
            .mouse_sensivity
            .lock()
            .unwrap();

        *s = (*s + delta*0.4).max(0.0);
    }

    pub fn decrease_mouse_sensitivity(&self, delta: f32)
    {
        let mut s = self
            .player_settings
            .mouse_sensivity
            .lock()
            .unwrap();

        *s = (*s - delta*0.4).max(0.0);
    }

}