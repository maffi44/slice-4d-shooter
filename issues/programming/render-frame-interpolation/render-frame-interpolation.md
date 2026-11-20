# Interpolate actor state between simulation ticks for rendering

- **Status:** Backlog — long-term
- **Priority:** Long-term
- **Complexity:** Complex
- **Created:** 2026-11-20
- **Stable address:** `issues/programming/render-frame-interpolation/`

## Goal

Render faster than the simulation with a real gain in smoothness: render frames between ticks should show actor positions (and other visual state) interpolated between the last two ticks instead of repeating the latest tick. Today render data changes once per tick, so frames above the tick rate only reduce the delay before a new tick appears ([configurable render rate](../render-frame-rate-setting/)).

## Open questions [open]

- What is interpolated: transforms of dynamic objects and the camera at least; materials, effects and UI probably not.
- Where: the world publishes the previous and current render data with tick timestamps, the render thread blends by its frame time; or a GPU-side blend.
- Latency: classic interpolation shows the past (up to one tick behind). For a competitive shooter the local player's camera may need extrapolation or render-time input instead, consistent with the minimal input-to-photon latency principle and the no-frame-queue rule ([refactoring context](../engine-refactoring-world-isolation/), §2.4).
- Teleports, spawns and removals must not be blended.
- Interaction with [tick scheduling](../engine-tick-rate/) and [GPU data grouping](../gpu-data-grouping/).

Not planned now; the author marked it as a very long-term, low-priority task.
