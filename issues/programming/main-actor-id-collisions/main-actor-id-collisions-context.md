# Context: Preserve main_actor_id during actor ID collisions

[Current task](main-actor-id-collisions.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

Review point 6. Author called this very important and high priority (2026-11-16).

The network peer ID may replace the main actor's initial ID; collision handling is deliberate, not merely a theoretical random-ID concern. Context: [engine refactoring](../engine-refactoring-world-isolation/), §20.2 and §24.7. Keep `ChangeMyId` semantics unless there is a separately agreed reason to change them.

## Implementation log

### 2026-11-20 — analysis

Defects found in the previous `src/engine/world/actors.rs`:

1. `insert`: an actor inserted with the main actor's ID displaced the main actor to a random ID, but `main_actor_id` stayed on the old ID and therefore pointed at the newly inserted actor (camera, HUD and `WorldContext::main_actor_id` would follow the wrong actor).
2. `change_id(old, new)` where `new == main_actor_id` and `old` is another actor: the main actor was displaced, `main_actor_id` kept `new` and identified the changed actor instead.
3. `change_id(id, id)` broadcast a spurious `IWasChangedMyId(id)`; `change_id(_, 0)` could give an actor the reserved outer-world sender ID, after which broadcast filtering by sender ID would treat it as the sender of every owner message.
4. `insert` accepted an actor that already carried ID 0.

The case "main actor changes to a taken ID" was already correct (`main_actor_id = new_id`, other actor displaced).

Queued references: direct messages and commands store IDs, and `Messenger` resolves them only at delivery. Rewriting queued IDs was considered and [rejected]: it would need a scan of every queue per change and would also redirect remote messages that are intentionally addressed to the new peer ID. The behavior is documented instead (main task, Result).

Notification ordering: in `change_id` the changed actor's notification (old -> new) is pushed before the displaced actor's (new -> random). With LIFO delivery a receiver such as `MoveWBonusSpot` (which rewrites a stored ID when `from` matches) first sees new -> random, then old -> new, and ends with the correct ID for either actor. With FIFO the same reference to `old` would be rewritten twice (old -> new -> random). No order of the two pushes is correct for both disciplines, because both notifications mention `new`; the dependency is therefore recorded in the message-ordering task rather than changed here.

### 2026-11-20 — implementation

- `Actors::place` (private) — common collision handling for `insert` and `change_id`, including the displaced main actor.
- `insert` treats `Some(0)` like `None` with a warning. `change_id` returns early for `new_id == 0` (error), `old_id == new_id`, and an unknown `old_id` (warning).
- Seven unit tests in `actors.rs` cover main/non-main collisions in both directions for `insert` and `change_id`, reserved/missing IDs, no-op changes, notification contents/order and ID validity. `Messenger::queued_broadcast_messages` is a `#[cfg(test)]` accessor for them.
- `docs/engine-architecture.md` documents the collision contract; the message-ordering task records the LIFO dependency.

Verification (Linux, uncommitted on top of `6361d4e`): release workspace build, workspace tests (including the seven new ones) and Windows GNU bins check pass without warnings. Headless and windowed runs of the default map (12 s and 20 s, offline) showed no errors or panics. The online path (`ChangeYourId` after connecting) was not exercised: it needs a local matchmaking/game server and a manual `G` press; the ID-change logic it calls is covered by the unit tests.

Closed at the author's request on 2026-11-20 together with the implementation.
