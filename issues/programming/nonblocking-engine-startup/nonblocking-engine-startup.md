# Responsive engine startup and MainLoop ownership

- **Status:** First implementation ready for author review — not yet accepted as done
- **Priority:** Normal
- **Created:** 2026-11-16
- **Stable address:** `issues/programming/nonblocking-engine-startup/`
- **Context:** [nonblocking-engine-startup-context.md](nonblocking-engine-startup-context.md)

## Goal

Keep the window responsive during GPU initialization and initial-world loading. Separate initialization ownership from the running Engine, expose MainLoop to the client entry point, and wake it on completion without empty simulation ticks.

## Accepted direction and first implementation

The author requested a separate asynchronous loader and authorized a first implementation for review:

```rust
let loading = engine::load(start_level, args.graphic_backend, args.headless);
MainLoop::run(loading);
```

- `EngineLoader` owns initialization lifetime, its worker and completion channel. Async map/configuration loading and GPU instance discovery run concurrently; a loading thread coordinates their dependencies and prepares input/audio/UI/shared systems. No startup handles or polling remain in Engine.
- First winit `resumed` creates the window. The loader requests surface creation on the main thread (required for Metal); adapter/device, resources, UI/upscale pipelines and world pipelines are prepared in the background.
- Loading uses `ControlFlow::Wait` and `EventLoopProxy` notifications after queued results. MainLoop consumes `LoadedEngine` through `Engine::new` only after required preparation succeeds, applies current dimensions and starts simulation deadlines.
- `WorldPreparation` retains CPU map data and shader inputs until GPU finalization. `WorldPreparation::load` and `WorldDescriptor::load` expose async loading: file I/O uses Tokio, while JSON/geometry parsing and shader/driver work stay on blocking workers/loading threads. Actor-driven transitions use the same async API.
- Settings are read asynchronously; one async writer serializes later saves and normal exit awaits pending writes. The I/O audit and justified synchronous exceptions are recorded in the context.
- Close during loading hides the window, cancels later stages and keeps dispatching until current work returns. Failures/panics terminate loading with an error. Headless and all three client bins use the same loader; headless never creates winit/GPU/rendering/sound devices.

## Scope and remaining limits

`LoadedEngine` is the completed result, distinct from the pending `EngineLoader`: Engine's constructor cannot accidentally wait on an unfinished handle. These names and the public entry shape are the first implementation's choices, open to author review.

Actor construction and world activation remain synchronous; this task does not implement the broader [async world preparation](../async-world-preparation/) resource/validation design. Window/surface creation still runs on the platform thread. Cancellation is cooperative between stages: a running driver call cannot be forcibly interrupted, so a hidden process can remain until it returns. MainLoop remains under `engine/` because it is the engine's application runner.

Do not add a heterogeneous task registry for two known startup branches. Future async work belongs to its actual resource/world/session owner. Existing running-state tick scheduling is unchanged ([tick scheduling](../engine-tick-rate/)).

## Acceptance and verification

OpenGL runtime verification on 2026-11-17 found a context-lock timeout after startup: a long render-thread submission conflicts with the first simulation uploads. The author subsequently chose to remove GL support and preserve asynchronous pipeline loading; see [OpenGL context contention](../opengl-context-contention/) for reproduction, decision and verification. GL is no longer a supported startup target.

The first implementation must compile all client entry points and preserve headless mode. Verify window responsiveness during GPU/shader loading, completion/error transitions, early close, latest resize at activation and normal shutdown. Duplicate resumes are guarded in code. Native Windows and Metal runtime checks remain required on those platforms; cross-compilation alone cannot establish driver/thread correctness.

Commands, runtime evidence and limitations are recorded in the context implementation log. Keep this task open until the author reviews the implementation and confirms completion.
