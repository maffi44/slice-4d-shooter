# Research surface presentation, frame pacing and graphics menu settings

- **Status:** Open — research pending
- **Priority:** Normal
- **Complexity:** Complex
- **Created:** 2026-11-20
- **Stable address:** `issues/programming/vsync-research/`

## Question

Can a bounded one-frame queue make camera motion as smooth at a lower render rate and GPU load as it is today at high FPS, while keeping input-to-photon latency acceptable? Examine queue depth alongside vertical synchronization and pacing to the display refresh rate.

The baseline uses a [manual target render frame rate](../render-frame-rate-setting/). The render scheduler skips the frame if the previous GPU submission is still busy. When a frame is due, it acquires the surface texture, submits once and calls `pre_present_notify` and `present` without waiting for that submission to finish. The surface requests `AutoNoVsync` and `desired_maximum_frame_latency: 2`. This setting alone does not tell us how many completed frames are waiting for display. [Render scheduler](../../../src/engine/shared_systems/render/render_scheduler.rs) · [Surface configuration](../../../src/engine/shared_systems/render/gpu_context.rs)

## What to research [open]

- Compare the baseline with allowing one additional frame to be submitted while its predecessor is in flight (two outstanding frame submissions at most). Verify the bound during a GPU slowdown and check whether older frames also accumulate on the presentation side. The author's earlier two-frame buffering trial is a qualitative reference. [Experiment context](vsync-research-context.md)
- Repeat the same camera turn and input action in the same scene at fixed resolution, quality, tick rate and display refresh rate. First compare both policies at the same FPS cap, then lower the cap to see whether either preserves smooth motion with less GPU work. Record displayed frame intervals and visible judder, input-to-visible-response delay and GPU utilization. Label timing proxies explicitly when actual display timing is unavailable.
- wgpu present modes on Vulkan, DX12 and Metal (`Fifo`, `FifoRelaxed`, `Mailbox`, `Immediate`, `AutoVsync`/`AutoNoVsync`): which can add a frame queue, where `get_current_texture` or `present` blocks, and how `desired_maximum_frame_latency` interacts with the render loop's "do not submit while the GPU is busy" check. Vsync can only lower or align the rate, never raise it above what the loop produces; the concern is added latency from queued frames.
- Whether `Mailbox` gives tear-free output without queue latency, and how widely it is available (Windows, Linux X11/Wayland, macOS).
- Reading the refresh rate: winit `MonitorHandle::refresh_rate_millihertz()` at window creation; several monitors, a window moved between monitors, variable refresh rate (FreeSync/G-Sync), unknown values.
- Alternative without vsync: set the target render frame rate from the detected refresh rate (an "auto" option in the setting).
- Relation to the observed judder at equal render and tick rates (beating between independent clocks) and to [frame interpolation](../render-frame-interpolation/).

## Graphics settings menu [open]

- Decide which choices a player benefits from seeing. Compare presets such as “Low latency” and “Smooth motion” with separate controls for presentation mode and maximum queued frames. Use the measurements above to explain what each choice does. Decide how a manual FPS cap, refresh-rate pacing and a VSync toggle interact, including which combinations the menu should offer.
- Assess a backend selector (for example, Vulkan and DirectX 12 on Windows). Determine which choices the current machine supports, how a saved preference interacts with the `--backend` override, and how the player recovers if the selected backend fails to start.
- Include display resolution, internal raymarch resolution scale and shadows in the proposed field list. Render scale and shadows already change through hotkeys. The loader currently passes the saved FPS target to the scheduler at startup, and backend selection takes place during GPU initialization. [Engine architecture](../../../docs/engine-architecture.md) · [Startup loader](../../../src/engine/loader.rs)
- For each proposed field, identify what applying it requires: a live value update, surface reconfiguration, rebuilding the GPU context and its dependent resources in the running client, or saving the setting and restarting the client. Trace two menu actions during a match: changing the FPS cap and switching from DX12 to Vulkan. For each, describe when the change takes effect, any interruption to the match, and which settings must be saved before restart.

## Expected result

Report the FPS cap, display refresh rate, backend and presentation mode for each comparison. Identify how much GPU load can be saved at comparable smoothness and what it costs in input latency. Use those results to recommend a queue depth and whether to keep the manual rate, add refresh-rate auto-detection or offer a vsync option. A change to the shipped policy requires a separate decision.

Produce a proposed graphics menu table listing each field, its player-facing choices and default, availability, and how changes take effect. Use it to agree on the settings contract before scheduling menu implementation.
