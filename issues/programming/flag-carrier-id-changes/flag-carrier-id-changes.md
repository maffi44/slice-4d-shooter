# Keep the captured flag attached when its carrier changes ID

- **Status:** Open — not implemented
- **Priority:** Normal
- **Complexity:** Simple
- **Created:** 2026-11-20
- **Stable address:** `issues/programming/flag-carrier-id-changes/`
- **Context:** [Review evidence and reasoning](flag-carrier-id-changes-context.md)

## Task

Update the flag's stored carrier identity when the world renumbers that actor.

## Possible solution [proposed]

Handle the existing notification in `Flag` when the captured ID matches the old ID. Update the carrier field in place so a rename does not replay capture effects or send another capture to the server.

## Completion criteria

Exercise a rename to a free ID and a collision with a second actor. The next position request and the death check must refer to the original carrier under its new ID. Unrelated renames and uncaptured flags keep their state.
