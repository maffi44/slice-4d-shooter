# Skip sound source construction when playback is unavailable

- **Status:** Done
- **Priority:** Long-term
- **Created:** 2026-11-20
- **Closed:** 2026-11-22
- **Resolution:** Done
- **Stable address:** `issues/programming/headless-sound-builder-allocation/`
- **Context:** [Review evidence and reasoning](headless-sound-builder-allocation-context.md)

## Task

Keep headless sound calls allocation-free before returning the empty source handle.

## Possible solution [proposed]

Check for a usable sound engine and a supplied buffer before constructing either builder or computing spatial parameters. Preserve the existing pending-buffer attachment path when both are present.

## Implementation [accepted]

`AudioSystem::spawn_source` takes a closure that builds the `SoundSourceBuilder`. It runs the closure only after checking the sound engine. Both public entry points return on an absent buffer first, and the spatial position conversion moved into the closure. With a sound engine, the pending-buffer path is unchanged.

The test `audio::tests::unavailable_playback_returns_before_building_a_source` covers this.

## Completion criteria

For spatial and non-spatial playback, verify that the absent-engine and absent-buffer paths return `None` before constructing a builder. Graphical playback with ready or pending buffers must retain its current behavior.
