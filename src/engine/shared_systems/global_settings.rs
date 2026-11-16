// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use std::{fs, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};


pub const USER_SETTINGS_FILE_NAME: &str = "user_settings.json";
pub const CONNECTION_SETTINGS_FILE_NAME: &str = "connection_settings.json";

/// The directory name inside the platform config directory
/// (Linux: ~/.config/slice4d, Windows: %APPDATA%\Slice4D, macOS: ~/Library/Application Support/Slice4D).
#[cfg(target_os = "linux")]
const CONFIG_DIR_NAME: &str = "slice4d";
#[cfg(not(target_os = "linux"))]
const CONFIG_DIR_NAME: &str = "Slice4D";

/// Compiled-in connection data. A new release can change servers or rotate TURN credentials
/// without stale values in the users' files.
const EMBEDDED_CONNECTION_SETTINGS: &str = include_str!("../../assets/settings/connection_settings.json");


/// Application-wide settings, part of the Engine Shared Systems.
///
/// - `user`: preferences the user can change; loaded from (and saved to) a user file.
///   Lookup order: next to the executable → the platform config dir → created there with defaults.
/// - `connection`: deployment data, compiled in; can be overridden only by
///   `connection_settings.json` next to the executable. Never saved.
pub struct GlobalSettings
{
    pub user: UserSettings,
    pub connection: ConnectionSettings,
    user_settings_file: Option<PathBuf>,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct UserSettings
{
    pub mouse_sensitivity: f32,
    pub master_volume: f32,
    pub screen_resolution_scale: f32,
    pub shadows_enabled: bool,
}

impl Default for UserSettings
{
    fn default() -> Self
    {
        UserSettings {
            mouse_sensitivity: 0.21,
            master_volume: 1.0,
            screen_resolution_scale: 1.0,
            shadows_enabled: true,
        }
    }
}


#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct ConnectionSettings
{
    pub matchmaking_server_url: String,
    pub ice_servers: Vec<String>,
    pub turn_server_username: String,
    pub turn_server_credential: String,
}


impl GlobalSettings
{
    pub fn load() -> GlobalSettings
    {
        let exe_dir = executable_dir();

        let (user, user_settings_file) = load_user_settings(exe_dir.as_deref());

        let connection = load_connection_settings(exe_dir.as_deref());

        GlobalSettings {
            user,
            connection,
            user_settings_file,
        }
    }

    /// Writes the user settings to the file they were loaded from.
    pub fn save(&self)
    {
        let Some(path) = &self.user_settings_file else {
            return;
        };

        match serde_json::to_string_pretty(&self.user)
        {
            Ok(json) =>
            {
                if let Err(e) = fs::write(path, json)
                {
                    log::warn!("global settings: can't save user settings to {}: {}", path.display(), e);
                }
            }
            Err(e) =>
            {
                log::warn!("global settings: can't serialize user settings: {}", e);
            }
        }
    }
}


fn executable_dir() -> Option<PathBuf>
{
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.to_path_buf()))
}


fn load_user_settings(exe_dir: Option<&Path>) -> (UserSettings, Option<PathBuf>)
{
    // 1. next to the executable
    if let Some(exe_dir) = exe_dir
    {
        let path = exe_dir.join(USER_SETTINGS_FILE_NAME);

        if let Some(settings) = read_json_file::<UserSettings>(&path)
        {
            log::info!("global settings: user settings loaded from {}", path.display());
            return (settings, Some(path));
        }
    }

    // 2. the platform config directory
    let Some(config_dir) = dirs::config_dir().map(|dir| dir.join(CONFIG_DIR_NAME)) else {
        log::warn!("global settings: no config directory on this platform, using default user settings");
        return (UserSettings::default(), None);
    };

    let path = config_dir.join(USER_SETTINGS_FILE_NAME);

    if let Some(settings) = read_json_file::<UserSettings>(&path)
    {
        log::info!("global settings: user settings loaded from {}", path.display());
        return (settings, Some(path));
    }

    // 3. create the user file with defaults
    let settings = UserSettings::default();

    if let Err(e) = fs::create_dir_all(&config_dir)
    {
        log::warn!("global settings: can't create {}: {}", config_dir.display(), e);
        return (settings, None);
    }

    let user_settings = GlobalSettings {
        user: settings.clone(),
        connection: ConnectionSettings::default(),
        user_settings_file: Some(path.clone()),
    };
    user_settings.save();

    log::info!("global settings: created {} with default user settings", path.display());

    (settings, Some(path))
}


fn load_connection_settings(exe_dir: Option<&Path>) -> ConnectionSettings
{
    if let Some(exe_dir) = exe_dir
    {
        let path = exe_dir.join(CONNECTION_SETTINGS_FILE_NAME);

        if let Some(settings) = read_json_file::<ConnectionSettings>(&path)
        {
            log::info!("global settings: connection settings overridden by {}", path.display());
            return settings;
        }
    }

    serde_json::from_str(EMBEDDED_CONNECTION_SETTINGS)
        .expect("ERROR: embedded connection_settings.json is not valid")
}


fn read_json_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Option<T>
{
    let content = fs::read_to_string(path).ok()?;

    match serde_json::from_str::<T>(&content)
    {
        Ok(value) => Some(value),
        Err(e) =>
        {
            log::warn!("global settings: can't parse {}: {}", path.display(), e);
            None
        }
    }
}
