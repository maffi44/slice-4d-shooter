# Configurable target render frame rate

- **Status:** Open — awaiting author review
- **Priority:** Normal
- **Complexity:** Simple
- **Created:** 2026-11-20
- **Stable address:** `issues/programming/render-frame-rate-setting/`

## Problem

[Capping the render rate at the 60 Hz simulation rate](../render-rate-limit/) made the game feel less smooth in the author's play test (2026-11-20), although the simulation was unchanged. Likely cause [inference, not measured]: the render loop and the simulation tick run on independent clocks, so at equal nominal rates the phase drifts and each tick is shown zero, one or two times irregularly (beating); the monitor refresh adds another unsynchronized clock. A faster render loop shows each new tick sooner and more evenly.

## Decision [accepted]

The author chose the simplest option: the target render frame rate is a user setting, editable by hand in `user_settings.json` and written back with the other settings. Automatic refresh-rate detection and vertical synchronization are a [separate research task](../vsync-research/); rendering faster than the simulation with real benefit needs [frame interpolation](../render-frame-interpolation/).

## Implementation — 2026-11-20

- `UserSettings::target_render_frame_rate: u32` (serde default 125 for files without the field; the field appears in the file on the next settings save).
- The loader passes it to `RenderScheduler::new`, which clamps it to 10..=1000 with a warning and keeps the fixed-timeline frame scheduling from the previous task. Read once at creation; no hotkey and no runtime change.
- README and `docs/engine-architecture.md` updated.

Default: the author first asked for 80 as an experiment, then set it back to 125 (the previous hard-coded render target) before committing, on 2026-11-20 [accepted]. The verification below ran with the 80 default; only the default value changed afterwards.

Verification (Linux, uncommitted on top of `181330f`): release workspace build, workspace tests and Windows GNU bins check pass without warnings; headless run without errors. Windowed `capture_the_flag_map` with the author's existing settings file (no field, default used): `target 80 FPS`, `avg dt 0.012515` (80 FPS), clean exit via Alt+F4. With a temporary `user_settings.json` next to the executable containing `5`: warning, clamped to 10, `avg dt 0.1000`. The temporary file was removed.

Completion: the setting controls the render rate, the default is 125, the author confirms the feel in play.
