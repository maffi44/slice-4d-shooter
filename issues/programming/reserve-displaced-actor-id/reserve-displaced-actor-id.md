# Exclude the displaced actor's old ID from replacement candidates

- **Status:** Done
- **Priority:** Long-term
- **Created:** 2026-11-20
- **Closed:** 2026-11-22
- **Resolution:** Done
- **Stable address:** `issues/programming/reserve-displaced-actor-id/`
- **Context:** [Review evidence and reasoning](reserve-displaced-actor-id-context.md)

## Task

Ensure displacement always chooses a different ID before the incoming actor occupies the requested slot.

## Solution [accepted]

`Actors::displace_actor_at` removes the current holder, then passes `Some(id)` to `get_new_random_uniq_id` to reserve the requested slot for its caller. Ordinary ID allocation passes `None`. The generator retries on either the excluded ID or an occupied ID. A free slot draws no candidates. See the [decision history](reserve-displaced-actor-id-context.md).

## Completion criteria

For insertion and rename collisions, both actors survive with distinct nonzero IDs and the main ID follows the same actor. Verify exclusion deterministically by construction or with controlled candidates. Repeated random trials cannot establish that property.
