# Closed tasks

Newest closure first. A task is listed here only after the author confirms its closure; resolutions are defined in the [issue README](README.md#closing-a-task). Open tasks: [open-tasks.md](open-tasks.md).

## Programming

| Task | Opened | Closed | Resolution |
|---|---|---|---|
| [Close connections when their owner is removed](programming/session-connection-lifetime/) | 2026-11-16 | 2026-11-23 | Done |
| [Keep the captured flag attached when its carrier changes ID](programming/flag-carrier-id-changes/) | 2026-11-20 | 2026-11-22 | Done |
| [Emit PlayerBeatLevel once per completed level](programming/single-level-completion-event/) | 2026-11-20 | 2026-11-22 | Done |
| [Resolve the root world's WorldFinished behavior](programming/root-world-finished/) | 2026-11-20 | 2026-11-22 | Done |
| [Skip sound source construction when playback is unavailable](programming/headless-sound-builder-allocation/) | 2026-11-20 | 2026-11-22 | Done |
| [Exclude the displaced actor's old ID from replacement candidates](programming/reserve-displaced-actor-id/) | 2026-11-20 | 2026-11-22 | Done |
| [Configurable target render frame rate](programming/render-frame-rate-setting/) | 2026-11-20 | 2026-11-20 | Done — commit `d7bcf17` |
| [Define and preserve message ordering](programming/message-ordering/) | 2026-11-16 | 2026-11-20 | Done — reviewed by the author |
| [Deliver broadcast messages by reference](programming/borrow-broadcast-messages/) | 2026-11-16 | 2026-11-20 | Done — reviewed by the author |
| [Preserve main_actor_id during actor ID collisions](programming/main-actor-id-collisions/) | 2026-11-16 | 2026-11-20 | Done — closed at the author's request without separate review |
| [Limit the render rate to the simulation rate](programming/render-rate-limit/) | 2026-11-20 | 2026-11-20 | Superseded — the 60 FPS cap felt worse in play; replaced by [configurable target render frame rate](programming/render-frame-rate-setting/) |
| [Prepare and validate worlds asynchronously](programming/async-world-preparation/) | 2026-11-16 | 2026-11-20 | Done — reviewed by the author; commit `858c5ae` |
| [Resource System](programming/resource-system/) | 2026-11-12 | 2026-11-20 | Done — reviewed by the author; commit `858c5ae` |
| [Responsive engine startup and MainLoop ownership](programming/nonblocking-engine-startup/) | 2026-11-16 | 2026-11-20 | Done — reviewed by the author; commit `76d73d9` |
| [OpenGL context contention during rendering](programming/opengl-context-contention/) | 2026-11-17 | 2026-11-20 | Done — OpenGL backend removed; reviewed by the author; commit `76d73d9` |
| [Engine refactoring: isolated, nestable worlds](programming/engine-refactoring-world-isolation/) | 2026-11-12 | 2026-11-16 | Done — accepted by the author; commit `caea9ec` |
