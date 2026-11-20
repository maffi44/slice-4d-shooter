# Preserve main_actor_id during actor ID collisions

- **Status:** Open — not implemented
- **Priority:** High
- **Created:** 2026-11-16
- **Stable address:** `issues/programming/main-actor-id-collisions/`
- **Context:** [main-actor-id-collisions-context.md](main-actor-id-collisions-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Task

`Actors::insert` and `Actors::change_id` may move an existing actor to a fresh ID when an incoming ID collides. When that displaced actor is the main actor, update `main_actor_id` too; the current code only covers part of this invariant.

Review insertion and ID changes in both directions, including main/non-main collisions and reserved external-sender ID 0. Preserve established ID-change notifications and clarify references already queued under old IDs.

Completion: the main ID always identifies the intended main actor after collision handling; actors are not lost, IDs stay valid and notification behavior is consistent. Source: `src/engine/world/actors.rs`, `src/actor.rs`.
