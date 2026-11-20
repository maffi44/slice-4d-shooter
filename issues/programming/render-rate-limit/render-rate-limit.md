# Limit the render rate to the simulation rate

- **Status:** Done — closed at the author's request on 2026-11-20 without separate review
- **Priority:** Normal
- **Created:** 2026-11-20
- **Closed:** 2026-11-20
- **Resolution:** Done
- **Stable address:** `issues/programming/render-rate-limit/`

## Problem

The render thread targets `TARGET_RENDER_FRAME_DURATION = 0.008` s (about 125 FPS) in `render_scheduler.rs`, while the simulation runs at 60 ticks/s (`GlobalTimeSystem::new(60)` in `shared_systems.rs`). Render data changes only once per tick and frames are not interpolated, so frames beyond the tick rate redraw identical data and spend GPU time for no visible gain.

## Task

When the render scheduler is created, derive its minimum frame duration from the simulation tick period (`shared.time.target_frame_duration`, available in the loader where `RenderScheduler::new` is called) instead of the hard-coded 125 FPS constant. The render rate must not exceed the simulation rate. Per-world `render_rate` (every Nth render frame) keeps its meaning.

Notes from the first analysis (2026-11-20, [proposed]):

- The render loop has its own clock. With 60 Hz on both sides and [tick scheduling](../engine-tick-rate/) lateness, render and simulation can beat against each other (occasional skipped or doubled tick). Waking the render thread when a tick publishes new data would avoid that, but it is a larger change; decide whether the simple cap is enough first.
- Keep the existing "do not submit while the GPU is busy" check (no frame queue).

Completion: the windowed client renders at most at the simulation rate (check the logged frame statistics), headless is unaffected, architecture documentation states the rule.

## Implementation — 2026-11-20 [accepted]

- `RenderScheduler::new` takes `min_frame_duration`; the loader passes `shared.time.target_frame_duration` (the 60 Hz tick period). The hard-coded `TARGET_RENDER_FRAME_DURATION` (8 ms with a 1 ms tolerance) is removed.
- Frames are scheduled on a fixed timeline (`next_frame_deadline += period`), so the average rate never exceeds the tick rate and lateness under one period is not accumulated. After a stall longer than one period the timeline restarts instead of catching up. The wake-on-tick alternative remains unimplemented; revisit it if render/tick beating becomes visible.
- `docs/engine-architecture.md` (render scheduler) states the rule.

Verification (Linux, uncommitted on top of `b39ec5b`): release workspace build, workspace tests and Windows GNU bins check pass without warnings; headless run is unaffected (15 s, no errors). Windowed run of `capture_the_flag_map` for about 20 s logged `render loop: avg dt 0.016669` (60 FPS; min 0.012, max 0.0227). The window was closed externally with `xdotool windowclose`, which made winit panic with an X11 `BadWindow` error at exit; that is an artifact of the external close, not of this change.
