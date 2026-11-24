# Lost surface recovery context

[Current task](lost-render-surface.md)

## 2026-11-24 — Dependency update review

[accepted] In wgpu 30.0.2, `CurrentSurfaceTexture::Outdated` calls for reconfiguring the existing surface. `Lost` calls for creating a new surface and configuring it. The native Vulkan backend can produce `Lost` for `ERROR_SURFACE_LOST_KHR`. Configuring the lost surface can fail again. This finding follows from the API and backend source. The review environment could not inject a real window-system loss.

Today `EngineLoader` asks the winit thread to create a surface for the instance before device selection. After startup the render loop owns the surface, configuration and pipelines. The startup request channel ends with the loader. The event-loop thread requirement for Metal is documented in [nonblocking startup](../nonblocking-engine-startup/).

[proposed] With the device still usable, allow one outstanding replacement request per affected window. Recheck format and size before the first presentation. A resize or close can arrive while the reply is in flight. Preserve simulation, slot ownership and the render thread's shutdown path. Decide how to report device loss separately, since replacing the surface alone cannot restore device resources.

## 2026-11-24 — Feasibility brief

Inspection at `a79eead` found one runtime surface owner, `RenderLoop`. World pipelines target the fixed `WORLD_OUTPUT_FORMAT` and retain their own output textures. Only the upscale pipeline and the three UI pipelines (progress bar, image, scanner) target the configured window format. The startup code has the original instance and selected adapter in scope but retains neither explicitly for recovery. Replacing a surface on the same window and device can preserve the running world, its GPU resources and network connections. The Complex estimate reflects thread coordination, loading and shutdown lifetimes, and backend verification.

[proposed] Give the graphical render thread a winit event proxy valid during splash and play. On loss, retire the old surface, send one window-thread creation request using the original instance and supply a sender for the render inbox. The event handler can then return the replacement directly to the requesting loop, including before the Engine is installed. Keep drawing suspended while a request is pending. Continue processing render messages and the existing idle-GPU upload flush. On close, discard the request or late reply. A blocking render-thread wait could deadlock with the scheduler's shutdown join.

[proposed] Check the new surface against the retained adapter and prefer the previous format. Use the current physical size, including during the splash when ordinary resize events are currently ignored. Defer configuration at zero size and reconcile any more recent resize before drawing. In the smallest version, losing support for the previous format produces a diagnostic and controlled application exit. An extended version could rebuild the four format-dependent pipelines while preserving UI buffers, textures and scanner bindings. Today the UI constructor also initializes resources through mutable access to the simulation's UI system. Reusing that entire constructor for format changes would cross the ownership boundary.

[open] Decide whether to support a format change and how many consecutive failed attempts to tolerate. One attempt per loss episode, reset after presentation resumes, gives a bounded first version. Surface creation failure, a replacement lost before presentation, an incompatible adapter, and a device-lost notification must reach the application. A panic confined to the render thread can leave simulation running until shutdown. A controlled exit on the first loss is cheaper but gives up the current session. Relaunch starts a new session. Recreating a lost device in-process would require a separate design for replacing every dependent GPU resource.

[proposed] Induce loss once during the splash and once during play, dropping the real surface and using the normal event and presentation paths. Verify a visible subsequent frame, preserved world and connection, resize to zero and restore, close with a reply outstanding, repeated loss and device failure. Record the backend. This checks application recovery, while a native driver-induced loss remains a separate verification result. This brief is based on source inspection only. See the [wgpu surface configuration contract](https://docs.rs/wgpu/30.0.2/wgpu/struct.Surface.html#method.configure).

## 2026-11-24 — Approved scope and implementation

[accepted] The author approved one replacement per loss episode on the same window, device and format. Creation failure, unsupported format, repeat loss before presentation, or device loss ends the graphical client with a diagnostic. The running engine saves settings on exit. Recreating the GPU device and converting the UI pipelines to a different format are outside this implementation.

[accepted] The loader registers the original instance and a render-inbox sender with the window thread before starting the render thread. This lets the window thread use a single FIFO for both replacement replies and resize events during loading and play. The renderer releases the lost handle before requesting another. While waiting it still handles shutdown, removes expired world slots and submits staged uploads on eligible idle-GPU frames at the existing 100 ms minimum interval. Close suppresses further creation requests. The renderer never waits synchronously for a window reply.

[accepted] Configuration takes place on the render thread before acquisition, including the first configuration. Size zero defers it. The old presentation format must appear in the replacement capabilities for the selected adapter. Automatic present, alpha and color-space choices keep their existing wgpu policy. Neither actors nor the loader’s resource clients acquire new GPU handles. A wgpu device-lost callback wakes the application independently of surface acquisition. The render thread also reports terminal errors, including native wgpu errors raised as panics. The wgpu 30 public `Queue::present` returns unit. Returning from that call is the application’s completion signal. A visible frame still needs checking on a display.

### Reproducing loss on a graphical host

Use a disposable build of `game-client-without-autoupdate` and a loopback-only connection configuration next to the executable. In `Presentation::acquire`, replace the expression `surface.get_current_texture()` in the match with the following expression. Only the acquisition result is injected. The old native surface is released and all subsequent creation, configuration and drawing use the production code.

```rust
if std::fs::remove_file("lose-window-surface").is_ok()
    || std::path::Path::new("repeat-window-loss").exists()
{
    CurrentSurfaceTexture::Lost
} else {
    surface.get_current_texture()
}
```

Place `lose-window-surface` in the working directory before a cold launch for splash recovery. During a local match, create the file again after moving the player. Observe the recovered view and verify that the local peer remains connected, with no second engine/world initialization. For consecutive loss, create `repeat-window-loss` while playing and check for one request and a failure exit. For close and resize, pause delivery of the replacement in a debugger, issue the window action and resume. Cover minimize and restore, closure without a blocked join, and upload-only submissions during an extended wait. To exercise device loss independently, temporarily call `device.destroy()` on an active test device and let the normal poll invoke its callback.

### Verification on Linux x86_64

Uncommitted implementation on `a79eead`, wgpu 30.0.2 and winit 0.30.13.

- Release workspace build, Linux workspace bins check and Windows GNU bins check passed without compiler warnings.
- Three surface-state tests passed in each of the three client bins. They exercise pending requests, eager release of the old handle, zero size and restore, applying the latest size once, reconfiguration on outdated status, repeat loss and a fresh attempt after presentation. A separate Vulkan probe using llvmpipe kept the queue alive, destroyed its device and observed the real `Destroyed` callback on poll.
- The full workspace tests ran with `--no-fail-fast`. Each client bin passed 32 tests. Its nine local socket tests and all 24 Matchbox socket tests failed at socket creation/bind with sandbox `EPERM`.
- The release headless client loaded the bundled CTF map and reached `engine: ready` with isolated settings. It was stopped after 12 seconds. The input subsystem logged a sandbox udev monitor restriction. Both launch checks used a loopback-only matchmaking override.
- The graphical Vulkan launch failed before window creation with `XOpenDisplayFailed`. Visible recovery on the splash and in a live match, close/resize races with the native window, upload behavior under prolonged native loss and complete application shutdown on device loss remain to be exercised on a graphical host.

## 2026-11-24 — Closure

[accepted] After a brief review, the author requested closure. The verification results and graphical-host reproduction procedure above were retained for future use.
