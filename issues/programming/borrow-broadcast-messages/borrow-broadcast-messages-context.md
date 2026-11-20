# Context: Deliver broadcast messages by reference

[Current task](borrow-broadcast-messages.md). Proposals below are not approved implementation decisions.

## 2026-11-16 — Author request

After accepting initial refactoring commit caea9ec, the author separately requested a small issue to avoid forgetting borrowed broadcast delivery. No elevated priority was specified.

[Refactoring context](../engine-refactoring-world-isolation/), §24.9 round 7 point 1, previously accepted broadcast clones because the message types were considered heap-free copies. Avoiding an allocation does not imply the whole-message copy is necessary. The new request targets those copies and API ownership; this is not a claim of measured performance gain.

The popped message is outside the Messenger buffers, so its immutable borrow can normally coexist with callbacks mutating Messenger to enqueue other work. Inspect handlers that currently consume message payloads and make only genuinely needed ownership transfers explicit. Queue ordering is a separate high-priority issue.

## 2026-11-20 — Implementation

Author requested implementation.

- `Actor::recieve_message(&mut self, message: &Message, ...)`; `actor_wrapper!` forwards the reference. Both broadcast and direct delivery in `Messenger::process_messages` pass `&message`; the popped value stays local for the duration of delivery. Queue order (LIFO) and the "skip the sender" filter are unchanged.
- All 17 handlers: `match &message.message` / `let message = &message.message`; `Copy` fields (ids, `Vec4`, `Transform`, `Team`, `FlagStatus`, numbers) are dereferenced where stored or passed on.
- Explicit ownership transfers: `PlayersDoll` clones `PlayerDollInputState` (five bools) for `respawn` and `SetNewTeamAndPosition`, because it stores it. `move_w_bonus::BonusSpotStatus` (enum around a `u128`) now derives `Copy`, like `flag::FlagStatus`.
- Before, each recipient received a full `Message` clone, including the `String` in `TriggerMessage::{Triggered, ActorEnteredTriggerArea}`; the trigger-name comparisons now borrow it.
- Commented-out legacy code in `diamond.rs` was left untouched.
- Architecture reference (Actors section) documents the borrowed callback contract.

Verification (Linux, uncommitted state on top of d7bcf17): `cargo check --workspace --bins --locked`, `cargo build --release --workspace --locked`, `cargo test --workspace --locked` (all passed), `cargo check --target x86_64-pc-windows-gnu --bins --locked` — no warnings. `game-client-without-autoupdate --headless` and the windowed client ran 10/15 s until terminated by `timeout` without panics. Gameplay message effects were not exercised online (no local server run); the change is type-checked mechanical borrowing.

## 2026-11-20 — Closure

The author reviewed the implementation and confirmed the task is done.
