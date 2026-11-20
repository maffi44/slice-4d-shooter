# Research vertical synchronization and display refresh rate

- **Status:** Open — research pending
- **Priority:** Normal
- **Created:** 2026-11-20
- **Stable address:** `issues/programming/vsync-research/`

## Question

Is vertical synchronization, or pacing the render loop to the display refresh rate, suitable for Slice, given that minimal input-to-photon latency is a core requirement and frame queueing was rejected ([refactoring context](../engine-refactoring-world-isolation/), §2.4)? The current approach is a [manual target render frame rate](../render-frame-rate-setting/) with `PresentMode::AutoNoVsync` and `desired_maximum_frame_latency: 2` (`gpu_context.rs`).

## What to research [open]

- wgpu present modes on Vulkan, DX12 and Metal (`Fifo`, `FifoRelaxed`, `Mailbox`, `Immediate`, `AutoVsync`/`AutoNoVsync`): which can add a frame queue, where `get_current_texture` or `present` blocks, and how `desired_maximum_frame_latency` interacts with the render loop's "do not submit while the GPU is busy" check. Vsync can only lower or align the rate, never raise it above what the loop produces; the concern is added latency from queued frames.
- Whether `Mailbox` gives tear-free output without queue latency, and how widely it is available (Windows, Linux X11/Wayland, macOS).
- Reading the refresh rate: winit `MonitorHandle::refresh_rate_millihertz()` at window creation; several monitors, a window moved between monitors, variable refresh rate (FreeSync/G-Sync), unknown values.
- Alternative without vsync: set the target render frame rate from the detected refresh rate (an "auto" option in the setting).
- Relation to the observed judder at equal render and tick rates (beating between independent clocks) and to [frame interpolation](../render-frame-interpolation/).

## Expected result

A recommendation with evidence (documentation, measurements on available platforms): keep the manual rate, add refresh-rate auto-detection, offer a vsync option, or a combination. No implementation in this task without a separate decision.
