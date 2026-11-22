# Keep the captured flag attached when its carrier changes ID

- **Status:** Done
- **Priority:** Normal
- **Created:** 2026-11-20
- **Closed:** 2026-11-22
- **Resolution:** Done
- **Stable address:** `issues/programming/flag-carrier-id-changes/`
- **Context:** [Review evidence and reasoning](flag-carrier-id-changes-context.md)

## Task

Update the flag's stored carrier identity when the world renumbers that actor.

## Possible solution [proposed]

Handle the existing notification in `Flag` when the captured ID matches the old ID. Update the carrier field in place so a rename does not replay capture effects or send another capture to the server.

## Implementation [accepted]

A captured flag keeps the server peer ID and an optional local carrier actor ID. It asks for the matching player on capture. A remote player that spawns later announces its peer ID. After binding, the flag sends position requests directly and follows the local actor through FIFO ID changes. Resolving or renaming a carrier leaves capture effects and server reporting alone.

The tests cover free and occupied renames, unrelated actors, an uncaptured flag, and joining before the carrier spawns under an occupied ID.

## Completion criteria

Exercise a rename to a free ID and a collision with a second actor. The next position request and the death check must refer to the original carrier under its new ID. Unrelated renames and uncaptured flags keep their state.
