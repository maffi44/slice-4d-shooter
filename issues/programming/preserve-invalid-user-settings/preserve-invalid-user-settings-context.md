# Context for preserve user settings when the FPS field is invalid

[Current task](preserve-invalid-user-settings.md)

## 2026-11-20 code review on `4007599`

The review reproduced this on Linux with the release client, an isolated platform config directory and no executable-local user settings. The file contained `{"master_volume":0.42,"target_render_frame_rate":-1}`. Startup logged a `u32` deserialization error and rewrote the file with volume `1.0` and FPS `125`. The same code accepts a missing FPS field through the serde default.

`read_json_file` turns a read or parse failure into `None`. When this happens for the platform file, `load_user_settings` takes the creation path. The render scheduler's range check happens after deserialization. A normal shutdown can also save a settings snapshot, so skipping only the startup write is insufficient for unrecoverable input. See [settings loading and saving](../../../src/engine/shared_systems/global_settings.rs) and the [render rate setting](../render-frame-rate-setting/).
