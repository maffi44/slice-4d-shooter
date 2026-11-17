# OpenGL context contention — investigation

Main task: [opengl-context-contention.md](opengl-context-contention.md).

## 2026-11-17 — Report and reproduction

The author reported a panic when launching the OpenGL backend. Investigation used the existing uncommitted startup implementation on `caea9ec`; unrelated working-tree changes were preserved. Linux/X11, kernel `6.12.10-76061203-generic`; the host has an NVIDIA GeForce RTX 3070 Laptop GPU and driver 570.172.08. The selected wgpu adapter was not logged, so host hardware alone is not proof of adapter selection.

1. Ran the existing `/tmp/slice-startup-check/window-check.py ready gl` harness against the release `game-client-without-autoupdate`. The window appeared in about 0.04 seconds, initialization reached `engine: ready`, then the main thread panicked. The harness subsequently failed its window-ping check.
2. Repeated with `RUST_BACKTRACE=1`. The stack identifies `Engine::tick → Queue::write_buffer → StagingBuffer::new → GLES Device::create_buffer → AdapterContext::lock`.
3. Inspected the locked dependency sources: wgpu-hal 27.0.4 `gles/egl.rs` uses `CONTEXT_LOCK_TIMEOUT_SECS = 1` in `try_lock_for`. `gles/queue.rs` holds this context lock across submission, GL command execution and flush. The WGL implementation also uses a one-second context-lock timeout, but Windows was not executed.
4. Temporarily logged before/after `queue.submit` and after `present`. A bounded 45-second release run logged a submission beginning at 13:57:58 UTC and completing at 13:58:22 UTC, after the main-thread panic. Earlier splash submissions returned promptly. The render thread then shut down; the process exited 101, rather than the timeout killing it. Log: `/tmp/slice-startup-check/gl-diagnostic.log`.

✅ Evidence: a GL submission holds the context longer than wgpu's one-second contention timeout; the concurrent upload panics. The observed submission eventually completes, so the diagnostic's deadlock suggestion is not sufficient evidence of deadlock here.

❓ Inference: deferred driver shader compilation/first-draw work is a plausible explanation for the long first world submission. The investigation did not profile the driver and does not establish which internal GL operation consumed those 24 seconds.

The temporary logs were removed, leaving `render_scheduler.rs` unchanged. The diagnostic client build passed without compiler warnings. No game protocol, dependency, version or production-service change was made. A complete engine validation suite is deferred until an implementation is selected; the present persistent changes are documentation only.

### Workaround verification

After removing diagnostic logging, rebuilt `game-client-without-autoupdate` in release mode with `--locked` without compiler warnings. The Vulkan window harness reached `engine: ready`, rendered the CTF view at 960×640 and closed normally with exit 0. It recorded 21 startup ping samples (maximum 5.33 ms; harness polling is not an input-latency benchmark). Artifacts: `/tmp/slice-startup-check/ready-vulkan.log` and `ready-vulkan.png`. No online-connect key was sent. `git diff --check` passed.

## Design boundary

The existing [startup design](../nonblocking-engine-startup/) preserves event-loop responsiveness by preparing resources off-thread. Simply putting the GL calls back on the main thread would sacrifice that behavior. Likewise, delaying the first simulation tick does not cover later world/pipeline creation or long submissions. A fix must coordinate all GPU callers, not only reorder the initial instance/surface handoff.

💡 Proposed to the author: one GPU owner thread, or a narrower GL-only synchronization fallback with an explicit window/simulation latency tradeoff. Neither option is accepted yet. The request for discussion follows AGENTS.md's requirement to discuss substantial architectural changes before implementation. The latest-state handoff proposal must be coordinated with [GPU data grouping](../gpu-data-grouping/), rather than silently resolving that task's open choices.

## 2026-11-17 — Requested explanation of the single GPU thread option

The author asked how this option differs from the current implementation; this is a request for explanation, not implementation approval.

💡 Extend the existing render thread's ownership to uploads, world/UI resource creation, pipeline creation and final GPU-resource release. Keep CPU simulation and CPU asset/map/shader-source preparation on their existing threads. Keep platform-required window/surface creation coordinated through MainLoop. Expose resource IDs/lifetime tokens and asynchronous requests outside the GPU thread, rather than clonable raw device/queue/buffer handles.

💡 Distinguish reliable ordered resource/lifecycle requests from replaceable render state. A bounded latest-state handoff with reusable CPU storage lets simulation advance while the renderer is busy; obsolete unconsumed visual states can be replaced, but creation/removal commands and gameplay events cannot be discarded. Consume a complete publication before uploads and submission. Per-world consistency follows from this; world/HUD consistency requires an explicit shared publication boundary. This does not require merging all shader bindings into a single GPU buffer.

Tradeoffs: additional CPU state storage and ownership/protocol complexity; snapshot publication determines which tick a frame sees; GPU pipeline creation on this thread can stall all rendering, even though the window/event loop remains responsive. A single blocking driver call cannot be time-sliced by the scheduler. Other backends can lose CPU parallelism during pipeline/resource creation if this policy is applied universally. Neither higher FPS nor lower latency has been measured or promised. Backend scope, publication boundary and scheduling of expensive preparation remain design choices for approval.

## 2026-11-17 — Author's decision: remove GL, preserve asynchronous loading

✅ The author rejected stopping rendering for background pipeline creation and maintaining separate architectures for GL and other backends. Given this raymarcher's hardware demands, he preferred removing GL and authorized implementation rather than preloading every shader or cancelling asynchronous loading. The initial allowed list was Vulkan, Metal and WebGPU; in response to a clarification he explicitly requested retaining DX12 too.

❌ Superseded: the earlier single-GPU-thread recommendation and GL-only synchronization proposal. They would trade loading/render responsiveness or architectural complexity for a backend the author no longer needs. Historical reasoning above is retained, but is no longer the implementation direction.

Technical correction: GL context affinity is to the thread where it is currently bound, not necessarily the main application thread. The inspected wgpu-hal implementation uses a lock and makes the context current on the caller's thread. Pipeline creation takes the same context lock as submission. A competing caller can panic after waiting one second while another driver call holds it. Background driver activity alone does not imply the context remains locked or that every compile will panic. Nor does this evidence imply that all other backends are free of driver stalls.

WebGPU in wgpu means the browser backend ([wgpu 27 Backend documentation](https://docs.rs/wgpu/27.0.1/wgpu/enum.Backend.html#variant.BrowserWebGpu)). It is allowed in the instance mask/features as requested; this does not restore the game's removed WebAssembly support or provide a native WebGPU selection.

### Implementation

- Explicit wgpu features retain Vulkan, DX12, Metal, WebGPU and the existing supporting features, excluding GLES. Cargo.lock was reconciled offline; comparison with HEAD confirmed no added/changed package versions. Eight now-unused GL dependency packages were removed.
- Instance creation uses an explicit allowed-backend mask and validates forced selections. It cannot discover GL automatically. Adapter logging now includes GPU name/backend.
- CLI retains Vulkan/DX12/Metal, rejects GL with an explanation before initialization, and explains native WebGPU unavailability. Invalid or missing backend values now correctly exit 2. All three clients share this parser.
- Removed the WebGL2-limit request fallback; default and downlevel device-limit requests remain. No shader, render scheduling, world ownership, message protocol or asynchronous-loading changes were made.
- Updated README, architecture documentation, AGENTS, issue index and the startup task. No commit, product-version bump or production-server connection.

### Verification — Linux/X11, uncommitted working tree based on caea9ec

- `cargo check --workspace --bins --locked`: passed without compiler warnings.
- `cargo build --release --workspace --locked`: passed without compiler warnings, including all three client binaries and servers.
- `cargo check --target x86_64-pc-windows-gnu --bins --locked`: passed without compiler warnings.
- `cargo test --workspace --locked`: passed outside the sandbox with local sockets, all 23 existing Matchbox integration tests. Client/server targets have zero tests; this does not establish gameplay coverage.
- Explicit Vulkan release run: reached ready, answered window events, and closed normally (exit 0). Logged NVIDIA GeForce RTX 3070 Laptop GPU (Vulkan). Its desktop screenshot was obscured by another application, so it is not image evidence.
- Automatic selection release run: also selected that GPU/Vulkan, reached ready and closed normally (exit 0). Inspected `/tmp/slice-startup-check/ready-auto.png`: CTF world and HUD rendered. Logs: `ready-vulkan.log` and `ready-auto.log` in the same directory. No pixel comparison claim; no shader change was made.
- Headless release run reached `engine: ready` without GPU/window/render initialization, ran for eight seconds, then was deliberately stopped by timeout (124).
- CLI checks passed for all three binaries: GL and uppercase GL rejection, browser WebGPU diagnostic, unknown/missing backend errors, help and recognition of Vulkan/DX12/Metal. All rejected backend cases exited 2 without engine initialization or panic.
- Cargo feature-tree inspection for Linux and Windows confirmed no GLES/WebGL features; Windows retains the DX12 feature. Cargo.lock package-version comparison passed. Documentation links and `git diff --check` passed.

Native Windows/DX12 and macOS/Metal execution remain untested on this Linux host; Windows compilation is not a runtime guarantee. Browser execution is outside the current product's supported targets. Task filenames remain open pending author acceptance.
