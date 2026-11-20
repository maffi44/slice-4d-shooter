# Do not silently ignore panics in background threads and tasks

- **Status:** Open — decision or implementation pending
- **Priority:** Normal
- **Complexity:** Complex
- **Created:** 2026-11-20
- **Stable address:** `issues/programming/unobserved-panics/`

## Problem

A panic outside the main thread is noticed late or never:

- Render thread (`render_scheduler.rs`): the main loop does not watch it. After a panic the game keeps simulating with a frozen picture; the panic is only logged from `RenderScheduler::drop` (`join()` returns `Err`).
- Tokio tasks whose `JoinHandle` is dropped (fire-and-forget): the Matchbox socket future and WebSocket reader/writer tasks in `shared_systems/net.rs`, the settings writer in `global_settings.rs`. A panic ends the task; the default panic message goes to stderr, which is invisible in the Windows GUI-subsystem `game-client`, and the owner may wait forever (for example, a connection that stays `Connecting`).
- Engine startup already converts loader panics and Tokio child failures into a terminal loading error ([startup task](../nonblocking-engine-startup/)); keep that behavior.

## Decisions needed [open]

Choose one policy for unexpected panics after startup, for example:

1. Fatal: a panic hook logs the message (through `log`) and sets a shared flag; the main loop checks it each tick and exits with an error. Compatible with the loader's `catch_unwind`.
2. Per subsystem: the render thread and each long-lived task report failure to their owner (the render scheduler, `NetSystem` marks the connection as lost, the settings writer reports through its flush path), and the owner decides.
3. `panic = "abort"` in release profiles: simplest, but it breaks the loader's `catch_unwind` and skips cleanup such as settings flushing.

Completion: no background panic can leave the game running in a broken state without a log entry visible on every platform; the chosen policy is documented in the architecture reference.
