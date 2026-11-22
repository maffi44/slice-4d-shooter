# Resolve the root world's WorldFinished behavior

- **Status:** Done
- **Priority:** Normal
- **Created:** 2026-11-20
- **Closed:** 2026-11-22
- **Resolution:** Done
- **Stable address:** `issues/programming/root-world-finished/`
- **Context:** [Review evidence and reasoning](root-world-finished-context.md)

## Task

Agree what the root owner does on `WorldFinished` and make execution and documentation consistent.

## Decision [accepted]

2026-11-22, the author: only `QuitApplication` exits the application. At the root, `WorldFinished` is ignored and the world keeps running. A nested world's owner chooses its own response. [rejected] Restoring the root exit from before `858c5ae`.

## Implementation

The `Engine::handle_outer_world_message` arm stays a no-op with a comment linking to this task. The enum comment and the architecture reference describe the agreed contract. The headless test `engine::tests::root_world_finished_keeps_running_and_quit_application_exits` uses in-memory settings and the default map, sends `WorldFinished` and checks that no exit is requested and the root world remains. It then checks that `QuitApplication` requests exit.

## Completion criteria

From a running root world, verify the chosen effect of `WorldFinished` and that `QuitApplication` still requests exit. Align the enum comment, architecture reference and verification with the agreed contract.
