# Resolve the root world's WorldFinished behavior

- **Status:** Open — decision pending
- **Priority:** Normal
- **Complexity:** Simple
- **Created:** 2026-11-20
- **Stable address:** `issues/programming/root-world-finished/`
- **Context:** [Review evidence and reasoning](root-world-finished-context.md)

## Task

Agree what the root owner does on `WorldFinished` and make execution and documentation consistent.

## Possible solution [proposed]

Restore the previous root exit behavior for compatibility, or confirm that only `QuitApplication` exits and update the architecture reference. Keep the nested-world owner free to choose its own response.

## Completion criteria

From a running root world, verify the chosen effect of `WorldFinished` and that `QuitApplication` still requests exit. Align the enum comment, architecture reference and verification with the agreed contract.
