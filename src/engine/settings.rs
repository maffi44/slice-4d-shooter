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