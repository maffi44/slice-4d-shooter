# Emit PlayerBeatLevel once per completed level

- **Status:** Done
- **Priority:** Normal
- **Created:** 2026-11-20
- **Closed:** 2026-11-22
- **Resolution:** Done
- **Stable address:** `issues/programming/single-level-completion-event/`
- **Context:** [Review evidence and reasoning](single-level-completion-event-context.md)

## Task

Make level completion a single notification and keep polling the destination independently.

## Possible solution [proposed]

Emit the event when the timer first crosses the completion threshold, or store whether this trigger has already sent it. Preserve the five-second transition delay, pending polling and single failure report.

## Implementation [accepted]

`FinalTrgger::advance_level_transition` broadcasts `PlayerBeatLevel` only on the tick when the timer first passes `LEVEL_COMPLETION_TIME` (3 s), by comparing the previous and new timer values. No extra flag is needed. The transition starting at `LEVEL_TRANSITION_TIME` (5 s) is unchanged: it polls a pending destination each tick, sends one `ReplaceRootWorld` when the destination is ready, and logs one error when it fails. The method takes `Messenger` and `ResourceSystem` instead of `EngineSharedSystems`, so tests can drive it.

Tests in `actor::final_trigger::tests` cover ready, delayed and missing destinations.

## Completion criteria

Advance a triggered actor across the threshold and several subsequent ticks. Count one completion broadcast for a ready, delayed or missing destination. During one successful transition exactly one replacement request is sent.
