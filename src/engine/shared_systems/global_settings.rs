// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use std::path::{Path, PathBuf};

use tokio::{fs, sync::{mpsc, oneshot}};

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
#[cfg_attr(test, derive(Default))] // In-memory settings with no persistence writer.
pub struct GlobalSettings
{
    pub user: UserSettings,
    pub connection: ConnectionSettings,
    pending_saves: Option<mpsc::UnboundedSender<SettingsWrite>>,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct UserSettings
{
    pub mouse_sensitivity: f32,
    pub master_volume: f32,
    pub screen_resolution_scale: f32,
    pub shadows_enabled: bool,
    /// Target (maximum) render frames per second. Edited by hand in the user file; read when the
    /// render scheduler is created. Higher than the 60 Hz simulation rate on purpose: without frame
    /// interpolation, a render loop capped at the tick rate visibly judders against the tick clock.
    pub target_render_frame_rate: u32,
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
            target_render_frame_rate: 125,
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


enum SettingsWrite
{
    Save(UserSettings),
    Flush(oneshot::Sender<()>),
}

impl GlobalSettings
{
    pub async fn load() -> GlobalSettings
    {
        let exe_dir = executable_dir();
        let (user, user_settings_file) = load_user_settings(exe_dir.as_deref()).await;
        let connection = load_connection_settings(exe_dir.as_deref()).await;
        let pending_saves = user_settings_file.map(start_settings_writer);

        GlobalSettings { user, connection, pending_saves }
    }

    /// Queues a snapshot. One writer preserves order; independent spawned writes could overwrite
    /// newer preferences with older ones. This never waits for disk I/O in a simulation/window callback.
    pub fn save(&self)
    {
        if let Some(writer) = &self.pending_saves
        {
            if writer.send(SettingsWrite::Save(self.user.clone())).is_err()
            {
                log::warn!("global settings: the settings writer has stopped");
            }
        }
    }

    /// Waits for earlier saves before the application's runtime is shut down.
    pub async fn flush(&self)
    {
        if let Some(writer) = &self.pending_saves
        {
            let (reply, done) = oneshot::channel();
            if writer.send(SettingsWrite::Flush(reply)).is_err() || done.await.is_err()
            {
                log::warn!("global settings: the settings writer stopped before flushing");
            }
        }
    }
}

fn start_settings_writer(path: PathBuf) -> mpsc::UnboundedSender<SettingsWrite>
{
    let (sender, mut receiver) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(command) = receiver.recv().await
        {
            match command
            {
                SettingsWrite::Save(user) => write_user_settings(&path, &user).await,
                SettingsWrite::Flush(reply) => { let _ = reply.send(()); }
            }
        }
    });
    sender
}

async fn write_user_settings(path: &Path, user: &UserSettings)
{
    match serde_json::to_string_pretty(user)
    {
        Ok(json) =>
        {
            if let Err(e) = fs::write(path, json).await
            {
                log::warn!("global settings: can't save user settings to {}: {}", path.display(), e);
            }
        }
        Err(e) => log::warn!("global settings: can't serialize user settings: {}", e),
    }
}


fn executable_dir() -> Option<PathBuf>
{
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.to_path_buf()))
}


async fn load_user_settings(exe_dir: Option<&Path>) -> (UserSettings, Option<PathBuf>)
{
    // 1. next to the executable
    if let Some(exe_dir) = exe_dir
    {
        let path = exe_dir.join(USER_SETTINGS_FILE_NAME);

        if let Some(settings) = read_json_file::<UserSettings>(&path).await
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

    if let Some(settings) = read_json_file::<UserSettings>(&path).await
    {
        log::info!("global settings: user settings loaded from {}", path.display());
        return (settings, Some(path));
    }

    // 3. create the user file with defaults
    let settings = UserSettings::default();

    if let Err(e) = fs::create_dir_all(&config_dir).await
    {
        log::warn!("global settings: can't create {}: {}", config_dir.display(), e);
        return (settings, None);
    }

    write_user_settings(&path, &settings).await;

    log::info!("global settings: created {} with default user settings", path.display());

    (settings, Some(path))
}


async fn load_connection_settings(exe_dir: Option<&Path>) -> ConnectionSettings
{
    if let Some(exe_dir) = exe_dir
    {
        let path = exe_dir.join(CONNECTION_SETTINGS_FILE_NAME);

        if let Some(settings) = read_json_file::<ConnectionSettings>(&path).await
        {
            log::info!("global settings: connection settings overridden by {}", path.display());
            return settings;
        }
    }

    serde_json::from_str(EMBEDDED_CONNECTION_SETTINGS)
        .expect("ERROR: embedded connection_settings.json is not valid")
}


async fn read_json_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Option<T>
{
    let content = fs::read_to_string(path).await.ok()?;

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

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn last_save_is_complete_before_runtime_shutdown()
    {
        let path = std::env::temp_dir().join(format!(
            "slice-settings-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
        ));
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let mut settings = GlobalSettings {
                user: UserSettings::default(),
                connection: ConnectionSettings::default(),
                pending_saves: Some(start_settings_writer(path.clone())),
            };
            // A burst of hotkey saves must not let an older write replace the latest one.
            for value in 0..32
            {
                settings.user.mouse_sensitivity = value as f32;
                settings.save();
            }
            settings.flush().await;
        });
        drop(runtime);

        let saved: UserSettings = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(saved.mouse_sensitivity, 31.0);
    }
}
