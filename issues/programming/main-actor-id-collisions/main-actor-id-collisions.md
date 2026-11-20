# Preserve main_actor_id during actor ID collisions

- **Status:** Done — closed at the author's request on 2026-11-20 without separate review
- **Priority:** High
- **Created:** 2026-11-16
- **Closed:** 2026-11-20
- **Resolution:** Done
- **Stable address:** `issues/programming/main-actor-id-collisions/`
- **Context:** [main-actor-id-collisions-context.md](main-actor-id-collisions-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Task

`Actors::insert` and `Actors::change_id` may move an existing actor to a fresh ID when an incoming ID collides. When that displaced actor is the main actor, update `main_actor_id` too; the previous code only covered part of this invariant.

Review insertion and ID changes in both directions, including main/non-main collisions and reserved external-sender ID 0. Preserve established ID-change notifications and clarify references already queued under old IDs.

Completion: the main ID always identifies the intended main actor after collision handling; actors are not lost, IDs stay valid and notification behavior is consistent. Source: `src/engine/world/actors.rs`, `src/actor.rs`.

## Result

- [accepted] One private helper `Actors::place` stores an actor under its ID and displaces the previous holder to a fresh random non-zero ID; `main_actor_id` follows the displaced actor when it was the main one. `insert` and `change_id` both use it. `change_id` additionally moves `main_actor_id` to `new_id` when the changed actor was the main one.
- [accepted] Reserved ID 0 (`OUTER_WORLD_SENDER_ID`): `insert` replaces a requested ID 0 with a random ID (warning); `change_id` to 0 is refused (error). `change_id` with an unknown old ID (warning) or with `old_id == new_id` changes nothing and sends no notification.
- [accepted] `ChangeMyId` semantics and `IWasChangedMyId` broadcasts are unchanged: the changed actor and a displaced actor each broadcast one notification from their previous ID.
- [accepted] Queued references are resolved by ID at delivery time; they are not rewritten. A direct message to a vacated ID is dropped; one to a reused ID reaches the new holder (this is how remote messages addressed by peer ID reach the renamed main player). Correct application of the two notifications of one collision by ID-tracking receivers relies on the current LIFO pump; this constraint is recorded in [message ordering](../message-ordering/).

Verification and details: implementation log in the context file.
