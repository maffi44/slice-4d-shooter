# Closed tasks

Newest closure first. A task is listed here only after the author confirms its closure; resolutions are defined in the [issue README](README.md#closing-a-task). Open tasks: [open-tasks.md](open-tasks.md).

## Programming

| Task | Opened | Closed | Resolution |
|---|---|---|---|
| [Preserve main_actor_id during actor ID collisions](programming/main-actor-id-collisions/) | 2026-11-16 | 2026-11-20 | Done — closed at the author's request without separate review |
| [Limit the render rate to the simulation rate](programming/render-rate-limit/) | 2026-11-20 | 2026-11-20 | Superseded — the 60 FPS cap felt worse in play; replaced by [configurable target render frame rate](programming/render-frame-rate-setting/) |
| [Prepare and validate worlds asynchronously](programming/async-world-preparation/) | 2026-11-16 | 2026-11-20 | Done — reviewed by the author; commit `858c5ae` |
| [Resource System](programming/resource-system/) | 2026-11-12 | 2026-11-20 | Done — reviewed by the author; commit `858c5ae` |
| [Responsive engine startup and MainLoop ownership](programming/nonblocking-engine-startup/) | 2026-11-16 | 2026-11-20 | Done — reviewed by the author; commit `76d73d9` |
| [OpenGL context contention during rendering](programming/opengl-context-contention/) | 2026-11-17 | 2026-11-20 | Done — OpenGL backend removed; reviewed by the author; commit `76d73d9` |
| [Engine refactoring: isolated, nestable worlds](programming/engine-refactoring-world-isolation/) | 2026-11-12 | 2026-11-16 | Done — accepted by the author; commit `caea9ec` |
