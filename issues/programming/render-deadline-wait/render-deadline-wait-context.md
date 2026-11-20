# Context for wait for the next render deadline without a fixed sleep ceiling

[Current task](render-deadline-wait.md)

## 2026-11-20 code review on `4007599`

`RenderLoop::run` sleeps for 3,122 µs after every iteration shorter than 3 ms. The loop needs at least one iteration per frame. Even with negligible work this bounds the fast path to about 320 iterations/s, although the setting accepts up to 1000 FPS. At 125 FPS the 8 ms deadline is sampled at sleep intervals. These bounds come from the code. Achieved FPS depends on the driver, GPU and OS scheduler.

`render_frame` already advances a fixed deadline and skips submissions while the GPU is busy. A late wake under one period is carried into the next deadline. See [scheduler](../../../src/engine/shared_systems/render/render_scheduler.rs), [fixed timeline](../render-rate-limit/) and [target FPS](../render-frame-rate-setting/).
