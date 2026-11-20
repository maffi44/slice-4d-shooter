# Context: Responsive engine startup and MainLoop ownership

[Current task](nonblocking-engine-startup.md). The initial research below is historical; the author's later separate-loader decision and implementation log follow it.

## 2026-11-16 — Author's request after accepting the first refactoring

The author committed the initial refactoring as `caea9ec` and considers that stage complete. He noticed startup can block after a window is created and suggested restoring explicit MainLoop construction/running in the client. Engine could be constructed in main, inside MainLoop.run, or on an initialization event. He wants parallel world and GPU loading, a responsive otherwise-idle loop, clean storage for current/future task handles, and no time-system machinery merely to check completion. A hash-based collection was an idea, not a requirement.

Historical decision: [refactoring context](../engine-refactoring-world-isolation/), §24.9 round 7 point 4, deliberately made `Engine::new(headless).run(...)` the entry API to remove experimental flags and redundant wrappers. Earlier §12.5 discussed loading execution; §9.8.1 required clean shutdown. Reopening the entry API clarifies application-loop versus simulation responsibilities, but the old API itself does not cause blocking. Keep the earlier simplifications and headless capability.

## Evidence from committed code and locked dependencies

Inspected `caea9ec`: `engine.rs`, `engine/main_loop.rs`, `shared_systems.rs`, `render/gpu_context.rs`, `render/render_scheduler.rs`, `world/world_descriptor.rs`, audio/UI initialization. Lock versions: winit 0.30.12, wgpu 27.0.1, Tokio 1.48.0.

GPU context creation currently includes Instance, window surface, adapter/device requests, configure, layouts and shared resources. RenderScheduler construction also creates UI GPU resources and pipelines on the caller. Descriptor loading consumes GPU and creates pipelines; calling `load(name, None)` and attaching GPU afterward is not a complete solution: shader name and allowed-dynamic-object settings currently live in parsing locals and must survive CPU preparation.

The installed wgpu 27.0.1 sources were checked: `src/api/surface.rs`, SurfaceTarget::Window documents a main-thread requirement on macOS/Metal; `src/backend/wgpu_core.rs`, request_adapter/request_device perform native work before returning ready futures. Therefore an async signature or polling a future on the UI thread does not guarantee nonblocking execution. Use blocking workers for eligible synchronous/driver work, rather than occupying Tokio's two ordinary workers with long blocking calls.

The installed winit 0.30.12 `src/application.rs` recommends creating windows/graphics after first resumed and tolerating duplicate resumes. `src/event_loop.rs` exposes EventLoopProxy and Wait/WaitUntil. Versioned online winit/wgpu pages were unavailable during research; installed pinned sources are the version-specific evidence. Accessible current winit docs describe the same wake-up mechanism (links below); they are not evidence of a dependency upgrade.

## Initial options and recommendation — superseded in part

Engine in main: explicit composition, easy headless ownership, avoids Option<Engine> solely because construction is hidden in an event. Recommended if construction is deliberately cheap. Engine in MainLoop.run before run_app: equally correct technically, but hides creation inside the runner. Engine in resumed: useful if the Engine fundamentally requires a window at construction, but then it needs absent/loading states and the callback must remain short. All three can be responsive or block, depending on work placement.

Separate `MainLoop::run` from Engine initialization conceptually. Engine remains responsible for startup dependencies; MainLoop owns platform event delivery. A proposed entry shape is `let engine = Engine::new(headless); MainLoop::new(engine).run(level, backend);` (sketch, not current API). Keeping the module under engine still matches its responsibility.

A fixed startup batch does not need a HashMap: heterogeneous result types require an enum or type erasure, and a map neither wakes the loop nor expresses task dependencies. Prefer a typed `Startup` object containing the coordinator/cancellation state; child JoinHandles and intermediate values can live in the coordinator future. The ordinary success path can send a typed Ready(Result<...>) event. Handle worker JoinErrors/panics as terminal failures as well; avoid fire-and-forget work that leaves the UI waiting forever.

If a genuinely dynamic batch of same-kind work appears, Tokio JoinSet<T> is an existing option (different outputs require a meaningful result enum). FuturesUnordered is for polling a dynamic set of futures and does not by itself spawn CPU work or wake winit; it is not needed for two known startup branches. Later actor/resource tasks belong to their resource/session/world owners, not automatically to an Engine-global map. Do not wait for all lifetime tasks before starting a world.

Suggested dependency flow: start CPU map preparation and eligible GPU preparation concurrently; create the OS window/surface on the main thread, then finish adapter/device/shared GPU preparation on a worker; join required CPU/GPU prerequisites; build map pipelines; report readiness; perform bounded main-thread activation. If Instance::new is itself slow, create it on the worker and request a surface through a typed main-thread event plus a one-shot reply before continuing GPU work. This keeps the required surface step on main without moving the whole GPU constructor there. Preserve the existing surface-compatible adapter choice.

During loading `ControlFlow::Wait` sleeps between events while handling close/resize and completion notifications. No TimeSystem is required for readiness. A worker reports its result through EventLoopProxy; user_event consumes it and forwards it to Engine. If using a separate result queue, publish before notifying and drain on wake. Do not synchronously await a task merely because it just sent a completion event: it may not have returned yet. Once ready, initialize simulation timing and use WaitUntil for ticks. An optional animated splash has its own rendering schedule and can only use the GPU once available; readiness does not need that animation clock.

Headless startup can await the same required preparation with no graphical branch, before entering its ordinary tick loop. Do not make headless depend on winit events.

## Shutdown and remaining constraints

Dropping JoinHandle detaches its task. Aborting a coordinator does not automatically stop its spawned blocking children. Started spawn_blocking work cannot be forcibly aborted, and Runtime drop can wait for it. A pending main-thread surface request must be completed or cancelled before its receiver/runtime is torn down, to avoid shutdown deadlock. Choose cooperative stage-boundary cancellation and ownership/cleanup explicitly; do not claim a timeout kills a driver call. Window/surface lifetime and main-thread destruction must remain valid even for late results after close. Test resize/minimize between surface creation and GPU completion using the current dimensions at activation.

Lightweight constructors and asynchronous loading are goals; current UI decoding, audio setup, UI pipeline construction and World::new are not proven cheap. This startup task and the already-open world-preparation task must agree which work can be prepared off-thread and which side effects are deferred to activation.

## Primary references

- [winit EventLoopProxy](https://docs.rs/winit/latest/winit/event_loop/struct.EventLoopProxy.html): cross-thread user-event delivery and error after loop closure.
- [winit ControlFlow](https://docs.rs/winit/latest/winit/event_loop/enum.ControlFlow.html): Wait versus WaitUntil/Poll.
- [winit ApplicationHandler](https://docs.rs/winit/latest/winit/application/trait.ApplicationHandler.html): resumed, user_event and lifecycle handling.
- [Tokio 1.48 spawn_blocking](https://docs.rs/tokio/1.48.0/tokio/task/fn.spawn_blocking.html): blocking work and shutdown limitations.
- [Tokio 1.48 JoinSet](https://docs.rs/tokio/1.48.0/tokio/task/struct.JoinSet.html): dynamic task groups with one output type.

At the initial research stage, no runtime experiment or startup code change was performed; these sources and code inspection support a proposal, not a tested cross-platform implementation.


## 2026-11-16 — Author's separate-loader proposal and authorization

[accepted] The author does not want Startup state/handles inside Engine. He proposed a function in the engine module returning an asynchronous loading object passed into MainLoop.run; this object owns task handles and dependency sequencing. When initialization finishes, Engine.new consumes the completed load. He authorized a first implementation if the design is sound, for subsequent review.

[rejected] Superseded: the earlier recommendation to construct a lightweight, partially initialized Engine in main with a Startup member. Its premise was that Engine should own initialization sequencing. The separate loader preserves explicit ownership while giving Engine a clearer invariant: required startup preparation is complete before it exists. The main loop still owns window/platform events; the loader owns loading, not simulation.

[accepted] Implementation refinement: the pending object is `EngineLoader`; its successful result is `LoadedEngine`. Passing the latter to `Engine::new` prevents an unfinished handle from being mistaken for ready state. `engine::load(level, backend, headless)` performs actual work (runtime creation and launching CPU/instance jobs), not a forwarding-only wrapper. MainLoop is externally callable and remains in `engine/main_loop.rs`.

## First implementation log

- Added `engine/loader.rs`. Two typed Tokio blocking-task handles are locals in the loading thread: CPU map preparation and, in graphical mode, GPU instance discovery. A named OS loading thread prepares shared systems and coordinates blocking work. This thread owns the runtime until it moves into the finished shared systems; destroying a runtime inside a Tokio async task would be incorrect. No generic task graph, HashMap, trait or new dependency was introduced.
- A small typed result channel carries surface requests and final success/failure. A one-time-installed winit proxy only wakes the loop: results are enqueued before notification. Installing the proxy also wakes once for any earlier results; a notification before `resumed` leaves requests queued until window creation. Headless receives directly from the same channel.
- Window and surface creation stay on the main thread; native Instance/adapter/device operations and texture/UI/upscale/world pipeline creation do not. `GpuContext::new` accepts an already-created instance/surface and captured dimensions. No unsafe cross-thread window/surface construction was added.
- Split map preparation from GPU finalization using a temporary `WorldPreparation` holding the parsed descriptor, shader name and allowed dynamic shapes. It is consumed by `finish(gpu)`; temporary shader inputs do not remain in the final descriptor. Existing `WorldDescriptor::load` composes the steps for current actor-driven level transitions. Map lookup, shaders, pipeline formats and gameplay are unchanged.
- MainLoop owns `Loading / Running / Stopped`. Only loading needs completion events; only running needs tick deadlines. The Engine has no pending initial descriptor, startup method, headless switch or run method. All three clients now create the loader and run MainLoop. `Engine::new` activates the initial world and resets tick timing after loading; current dimensions are reapplied before scheduling ticks.
- During loading, gameplay input is ignored rather than accumulated. Closing hides the window and requests cancellation; the loop keeps servicing pending surface/completion messages until resources are released. Shutdown never assumes `abort` can kill started driver work. Loader Drop disconnects/drains its receiver before joining so an unanswered surface request cannot deadlock teardown. MainLoop retains the final main-thread window owner through both success and cancellation.
- Loader panics and Tokio child failures are reported as terminal initialization errors; the current application reports them through its existing panic/error path rather than introducing a new error-screen subsystem. A failed surface reply cancels the worker. A missing map must not leave a permanent loading window.
- Removed inaccurate “World::new is cheap” comments in touched architecture code: actor constructors, callbacks and per-world resources still run during activation. Full asynchronous actor/resource preparation is deliberately left in [its high-priority task](../async-world-preparation/). Tick-rate corrections and general input-state changes remain separate too.
- Updated AGENTS, README, the architecture reference and issue index for the implemented entry API and loading lifetime. No commit, dependency/version change or production connection.

### Verification (Linux/X11 working tree based on caea9ec)

- `cargo check --workspace --bins --locked`: passed without compiler warnings.
- `cargo build --release --workspace --locked`: passed without compiler warnings.
- `cargo check --target x86_64-pc-windows-gnu --bins --locked`: passed without compiler warnings; this is compilation, not Windows runtime verification.
- `cargo test --workspace --locked`: all 23 existing Matchbox integration tests passed outside the sandbox (sandbox denies local socket bind). Client/server targets currently have zero tests; this result does not establish gameplay correctness.
- Headless release run with isolated config: loaded CTF and logged `engine: ready`; ran for 8 seconds, then deliberately stopped by timeout (124). Outside the sandbox, input initialization had no sandbox udev failure. No GPU/window/render initialization logged.
- First graphical Vulkan run: window appeared in 0.144 s; 130 `_NET_WM_PING` replies sampled during loading, maximum observed round trip 5.4 ms (harness polling interval 5 ms, not an input-latency benchmark). Reached the playable CTF view; screenshot inspected; normal WM_DELETE_WINDOW shutdown exited 0. Artifacts in `/tmp/slice-startup-check/`. Initial resize request was overridden by fullscreen/WM timing, so that run alone does not verify changed dimensions.

Additional checks are recorded below. macOS/Metal and native Windows runtime execution are unavailable on this Linux host. Required main-thread surface creation is preserved by code inspection; no cross-platform runtime guarantee is claimed.


### Final checks — 2026-11-17

- Early close during Vulkan loading (before `engine: ready`): the window answered a ping in 5.14 ms; WM_DELETE_WINDOW cancelled startup and the process exited 0 without activating the world.
- Invalid map in graphical Vulkan mode: the window continued answering pings (21 samples, observed maximum 19.24 ms), then exited with an explicit `Engine loading failed` message and stopped the render thread. It did not remain in loading state.
- Final Windows GNU bins check and `git diff --check` passed. Release binaries include the final channel/type cleanup.
- Remaining manual coverage: GL, native Windows/DX12, macOS/Metal; actual resize/minimize during loading (the first fullscreen resize attempt did not change dimensions), repeated OS resume, and a before/after screenshot comparison. The current Vulkan image was inspected, but no baseline image comparison was performed. These limits do not change the first implementation's review-ready status.

### Backend follow-up — 2026-11-17

Subsequent GL verification reproduced a context-lock timeout during a long first world submission. The author chose removal of GL support while retaining Vulkan, DX12, Metal and browser WebGPU in the backend policy; asynchronous pipeline/resource loading remains unchanged. See [the backend task](../opengl-context-contention/) for the decision, implementation and full checks. GL runtime coverage is therefore no longer an acceptance requirement. The subsequent automatic-selection Vulkan run rendered at 960×640 and exited normally; explicit Vulkan, headless, workspace tests/release build and Windows GNU compilation also passed.

## 2026-11-17 — Async I/O audit and implementation

[accepted] The author requested async resource/I/O APIs wherever they fit the engine without disproportionate complexity, and authorized implementation plus an AGENTS rule. The earlier “put the entire load on a blocking worker” approach was safe for responsiveness but exposed an unnecessarily synchronous API to its callers. Resource I/O now composes with other futures; synchronous CPU/native work remains explicitly isolated.

### Findings

- `spawn_blocking` dispatches a closure to Tokio's blocking pool; it does not just mark the current async worker as allowed to block. This is distinct from `block_in_place`. Blocking jobs do not yield simply because the runtime knows about them.
- Tokio 1.48 file I/O itself uses `spawn_blocking` internally on supported platforms. `tokio::fs` provides an awaitable contract; switching to it does not imply kernel-native asynchronous disk access or an automatic performance improvement. Use whole-file operations for these small/medium assets to avoid one job per read chunk. [Versioned Tokio fs documentation](https://docs.rs/tokio/1.48.0/tokio/fs/index.html).
- Heavy synchronous computations also belong outside normal async workers: JSON/geometry parsing, BSP shader generation, image/sound decoding and pipeline creation do not yield to the scheduler. [Tokio spawn_blocking documentation](https://docs.rs/tokio/1.48.0/tokio/task/fn.spawn_blocking.html).
- Installed wgpu 27.0.1 `src/backend/wgpu_core.rs`, `request_adapter` and `request_device`, performs core/native operations and returns `Box::pin(ready(...))`. The public async signature supports different backends but does not make native driver work cooperative. Keep polling the GPU constructor on the dedicated loading thread. Surface creation still belongs on the event-loop thread for Metal.

| Inspected operation | Execution after this change | Reason |
|---|---|---|
| Map candidate reads | `tokio::fs::read_to_string(...).await` | Awaitable I/O, unchanged lookup order/fallbacks |
| Embedded map fallback | Borrowed static string, no file I/O | `include_str!` data is already in the executable |
| Large map JSON/geometry parsing | `spawn_blocking`, awaited by preparation | CPU work must not occupy either of the two async workers |
| World shader/pipeline preparation | Loading thread at startup; `spawn_blocking` for transitions | Synchronous CPU/driver work |
| User/connection settings reads and default-file creation | Async Tokio file/directory operations | Small JSON parsing remains inline; it does not warrant separate CPU jobs |
| Settings saves | One async writer, ordered snapshots; final flush on normal exit | Avoid disk I/O in tick/window callbacks without stale-write races |
| Audio/UI initialization | Dedicated loading thread | Embedded bytes, synchronous decode/device APIs; no disk reads to convert |
| Input initialization | Dedicated loading thread | Gilrs platform setup; existing event collection remains synchronous |
| NetSystem | Existing async socket tasks retained | Ticks consume queues/state; transport already uses runtime tasks |
| MainLoop/Engine simulation | Synchronous | Event delivery and simulation are not resource-loading futures |
| Executable/config path discovery | Small synchronous platform queries | No useful async replacement; file contents are read asynchronously |

### Changes and boundaries

- `WorldPreparation::load` and `WorldDescriptor::load` are now async. The former reads the first readable candidate asynchronously (preserving fallback on read errors), borrows the embedded fallback without copying it, then awaits CPU parsing. The latter also awaits GPU preparation on a blocking worker.
- Initial loading spawns the map and `GlobalSettings::load` futures immediately, alongside GPU instance discovery. Shared systems receive loaded settings rather than opening files in their constructor. The short-lived OS loading thread remains: it already runs substantial synchronous native/decode work and safely owns/moves the runtime. `block_on` at this synchronous boundary waits on the async work; it does not occupy an ordinary runtime worker.
- `FinalTrgger` now spawns the async descriptor API. Its existing explicit `async_load = false` transition mode still blocks at the trigger's caller boundary; changing that gameplay/map contract is outside this I/O update. Async mode only consumes the JoinHandle after `is_finished`.
- `GlobalSettings::save` queues a four-field snapshot; a single task awaits writes in order. `flush` acknowledges all prior writes, and normal graphical/headless exit waits after the final save. This guarantees completion of file-write operations, not power-loss durability or persistence after forced process termination. Serialization/file failures retain warning-based handling.
- Added one regression test for a burst of 32 saves, flush and immediate runtime shutdown: the resulting file must contain the latest snapshot. It runs once for each of the three client bins. No general test-suite initiative or new dependency was introduced.
- Explicitly enabled Tokio `fs` and `sync` features without version updates; preserved the already-present backend policy/lockfile changes from the separate OpenGL work.
- Updated AGENTS, README, architecture and this task's current summary. The new rule prefers async I/O with concrete lifecycle/performance exceptions rather than requiring `async fn` on every constructor.

### Verification

- Linux `cargo check --workspace --bins --locked`, release workspace build and Windows GNU bins check passed without compiler warnings.
- Workspace tests passed: 23 Matchbox integration tests plus the new settings-persistence regression in each of the three client bins (26 executions). Socket tests ran outside the sandbox.
- Headless release run reached `engine: ready` with asynchronous map/configuration loading; deliberately stopped after 8 seconds by timeout (124).
- Graphical Vulkan reached the rendered CTF world at 960×640 and exited normally (0), including the final settings flush. The window appeared in 0.107 s; 21 startup ping samples had a maximum observed round trip of 5.13 ms (5 ms sampling granularity, not a latency benchmark).
- `git diff --check` passed. Native Windows/macOS execution and actor-driven map transitions were not runtime-tested in this pass; all affected entry points/actor code compile. No production connection or commit.

## 2026-11-20 — Closed

[accepted] The author confirmed that the review was done and the implementation committed in `76d73d9`; the task is closed as Done. Later changes go to new or existing open tasks.
