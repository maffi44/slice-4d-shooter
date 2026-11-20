# Emit PlayerBeatLevel once per completed level

- **Status:** Open — not implemented
- **Priority:** Normal
- **Complexity:** Very simple
- **Created:** 2026-11-20
- **Stable address:** `issues/programming/single-level-completion-event/`
- **Context:** [Review evidence and reasoning](single-level-completion-event-context.md)

## Task

Make level completion a single notification and keep polling the destination independently.

## Possible solution [proposed]

Emit the event when the timer first crosses the completion threshold, or store whether this trigger has already sent it. Preserve the five-second transition delay, pending polling and single failure report.

## Completion criteria

Advance a triggered actor across the threshold and several subsequent ticks. Count one completion broadcast for a ready, delayed or missing destination. During one successful transition exactly one replacement request is sent.
