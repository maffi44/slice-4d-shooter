# Exclude the displaced actor's old ID from replacement candidates

- **Status:** Open — not implemented
- **Priority:** Long-term
- **Complexity:** Very simple
- **Created:** 2026-11-20
- **Stable address:** `issues/programming/reserve-displaced-actor-id/`
- **Context:** [Review evidence and reasoning](reserve-displaced-actor-id-context.md)

## Task

Ensure displacement always chooses a different ID before the incoming actor occupies the requested slot.

## Possible solution [proposed]

Generate a replacement while the displaced slot is still occupied, or explicitly reject the vacated ID when choosing a candidate. Keep the no-collision path free of random-ID generation.

## Completion criteria

For insertion and rename collisions, both actors survive with distinct nonzero IDs and the main ID follows the same actor. Verify exclusion deterministically by construction or with controlled candidates. Repeated random trials cannot establish that property.
