# Engine refactoring: isolated, nestable worlds

- **Status:** ✅ Done — accepted by the author on 2026-11-16
- **Priority:** Normal
- **Created:** 2026-11-12
- **Completed:** 2026-11-16
- **Stable address:** `issues/programming/engine-refactoring-world-isolation/`
- **Context:** [done-2026-11-16-engine-refactoring-world-isolation-context.md](done-2026-11-16-engine-refactoring-world-isolation-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Current state and scope

Implemented on `dev` and accepted by the author as complete on 2026-11-16. The review follow-ups below are separate tasks, not part of this one. No new engine redesign is authorized by this task.

- `Engine` owns the root world and `EngineSharedSystems`; actors own nested worlds.
- Each world owns actors, messenger, physics, time and rendering state.
- A shared render thread schedules worlds; `NetSystem` transports packets and `SessionController` owns game-session logic.
- Settings are split into persisted user preferences, connection settings and gameplay constants.
- Entry point: `Engine::new(headless).run(level, backend)`; all three client bins remain supported.

The maintained implementation reference is [Engine architecture](../../../docs/engine-architecture.md). Historical design and implementation evidence are in the context: Part I (rationale), §19 (discussion), §24.9 (implementation log), §24.10 (manual verification). Historical Rust sketches are not a current API specification.

## Review follow-ups (2026-11-16)

High priority: [connection ownership](../session-connection-lifetime/), [message ordering](../message-ordering/), [main actor ID collisions](../main-actor-id-collisions/), [async world preparation](../async-world-preparation/).

Other agreed tasks are indexed in [issues](../../README.md). GPU data grouping and overflow protection are long-term work. The author did not request separate tasks for a nested-world test scenario, performance instrumentation or a test suite at this stage. Existing allocator and tick-rate issues remain separate.

## Acceptance

The author accepted the refactoring as complete on 2026-11-16. The list below was the manual-review checklist at the time of acceptance; it is kept for reference. Any problem found later goes into a new or existing task, not back into this one.


- Author's gameplay review: movement, weapons, holes, scanner, flags, bonuses, respawn and HUD.
- Local two-client networking review, including disconnect/reconnect; never use production in automated runs.
- Settings persistence, resize/fullscreen and Windows updater behavior.
- Supported obstacle-course maps and transitions remain affected by [map compatibility](../maps-out-of-sync-with-parser/).
- Performance assessment on the author's gaming hardware.
- Agree how review follow-ups affect final acceptance; none is marked fixed by this documentation migration.

## Verification evidence

The 2026-11-16 review ran workspace check, release workspace build, Windows GNU bins check and workspace tests successfully on the uncommitted tree. The 23 tests belong to vendored Matchbox. Headless CTF ran for eight seconds until timeout, with a udev-monitor error. Windowed behavior and two-client gameplay were not verified in that review. Earlier runs and their limitations remain in context §24.9.

Renamed to the completed filenames on 2026-11-16 after the author's explicit acceptance.
