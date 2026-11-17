# OpenGL context contention during rendering

- **Status:** OpenGL removal implemented and verified on Linux — awaiting author acceptance
- **Priority:** Normal
- **Created:** 2026-11-17
- **Stable address:** `issues/programming/opengl-context-contention/`
- **Context:** [opengl-context-contention-context.md](opengl-context-contention-context.md)

## Problem

Launching the client with `-b gl` could panic after `engine: ready` with `Could not lock adapter context. This is most-likely a deadlock.` Simulation uploads and render-thread submission compete for the same GL context. Installed wgpu-hal 27.0.4 has a one-second context-lock timeout; a reproduced submission held the context for approximately 24 seconds and eventually returned. Pipeline creation also holds this lock. This is not a requirement for GL to run on the application's main thread, nor evidence that every asynchronous driver operation causes a panic.

## Accepted decision — 2026-11-17

✅ The author chose to remove OpenGL support instead of serializing all GPU work or introducing separate GL/non-GL architectures. Preserve asynchronous resource/pipeline loading: making rendering wait for background pipeline creation is unacceptable. Do not require preloading every game shader at startup.

✅ Allowed instance backends: Vulkan, DX12, Metal and browser WebGPU. The author explicitly clarified that DX12 should remain supported. Native CLI choices are `vulkan`, `dx12`, `metal`; browser WebGPU is not available in the current native client, and this change does not restore the removed WebAssembly build.

❌ Superseded proposals: a single GPU owner thread and GL-only synchronization. Their investigation and tradeoffs remain in the context. No GPU ownership rewrite or change to [GPU data grouping](../gpu-data-grouping/) is part of this fix.

## Implementation

- Disable wgpu default features and explicitly retain `std`, `parking_lot`, `vulkan`, `dx12`, `metal`, `webgpu`, `wgsl`. Remove unused GL dependency entries from Cargo.lock without upgrading remaining packages.
- Replace `Backends::all()` with the explicit allowed set; validate explicitly requested backends against it too. GL cannot be selected automatically or through the loader API.
- Reject `-b gl` before engine/window initialization with an explanatory error and exit code 2. Report browser-only WebGPU clearly. Unknown/missing backend arguments also exit 2 instead of success.
- Remove the WebGL2 device-limit fallback; retain the existing default/downlevel attempts. Log the selected adapter name and backend.
- Update README, architecture, agent guidance and related startup task. Preserve current multi-threaded loading/rendering and headless behavior.

## Completion criteria

- Release workspace build, workspace tests and Windows GNU bins check pass without compiler warnings.
- Explicit Vulkan and automatic backend selection reach the rendered CTF world and close normally; inspect the image. Headless still loads without GPU/window initialization.
- CLI rejects GL without initializing the engine; help lists supported native backends. Cargo feature inspection confirms GLES/WebGL is not enabled and DX12 remains enabled on Windows.
- Record verification and native-platform coverage limits in the context. Keep this task open until the author's completion confirmation.
