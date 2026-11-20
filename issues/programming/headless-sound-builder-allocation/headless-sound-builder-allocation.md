# Skip sound source construction when playback is unavailable

- **Status:** Open — not implemented
- **Priority:** Long-term
- **Complexity:** Very simple
- **Created:** 2026-11-20
- **Stable address:** `issues/programming/headless-sound-builder-allocation/`
- **Context:** [Review evidence and reasoning](headless-sound-builder-allocation-context.md)

## Task

Keep headless sound calls allocation-free before returning the empty source handle.

## Possible solution [proposed]

Check for a usable sound engine and a supplied buffer before constructing either builder or computing spatial parameters. Preserve the existing pending-buffer attachment path when both are present.

## Completion criteria

For spatial and non-spatial playback, verify that the absent-engine and absent-buffer paths return `None` before constructing a builder. Graphical playback with ready or pending buffers must retain its current behavior.
