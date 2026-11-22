# Context for exclude the displaced actor's old ID from replacement candidates

[Current task](reserve-displaced-actor-id.md)

## 2026-11-20 code review on `4007599`

`Actors::vacate` removes the old holder before `get_new_random_uniq_id` checks the occupied keys. If the generator returns that same ID, the displaced actor is reinserted there and overwritten by the incoming actor. If it was the main actor, the camera ID then identifies the incoming actor. This is a code-level counterexample with negligible probability for a random `u128`.

Both insertion and `ChangeMyId` use this helper. During `change_id`, the changing actor's source ID is already kept occupied while the destination is vacated. Maintain that protection and the displaced-first FIFO notifications. See [actor storage](../../../src/engine/world/actors.rs) and [main actor ID collisions](../main-actor-id-collisions/).

## 2026-11-22 implementation [superseded]

Chosen variant: generate the replacement while the displaced slot is still occupied, instead of passing an explicit excluded ID. Reason: the existing occupancy check already rejects it, so exclusion holds by construction and needs no extra parameter or comparison. The alternative (remove first, then reject the vacated ID explicitly) was not taken because it adds a special case that every future caller must remember.

- `vacate` checks `contains_key` first; a free ID returns without drawing candidates, so the no-collision path stays free of random-ID generation (same single lookup as before).
- On a collision it calls `new_unique_id` before `remove`, then moves the actor. The collision path costs one extra hash lookup, which is negligible and happens only on collisions.
- `get_new_random_uniq_id` became `new_unique_id(next_candidate)`, and `vacate` takes `next_candidate` directly. Production callers (`insert`, `change_id`) pass `get_random_non_zero_id`; tests pass a fixed sequence. Both insertion and `ChangeMyId` go through `vacate`, so the shared helper is the one tested.
- `change_id` still vacates while `old_id` is occupied, and the displaced-first FIFO notification order is unchanged.

Tests in `src/engine/world/actors.rs`: the candidates `[10, 20, 30]` while vacating 10 must yield 30 for the insertion path and the rename path (`change_id(20, 10)`, where 20 is still held), and the main ID must follow the displaced actor. Vacating a free ID with an empty candidate list must not draw. With the old order (remove, then generate) the first candidate 10 would have been accepted.

Verification on uncommitted changes over `ce44b8b` (Linux): `cargo build --release --workspace --locked` with no warnings, `cargo test --workspace --locked` with all tests passing, and `cargo check --target x86_64-pc-windows-gnu --bins --locked` with no warnings. Headless and windowed runs were skipped: the change is limited to the actor container logic covered by unit tests, and the collision is impossible to trigger reliably in play.

## 2026-11-22 review: remove the forwarding wrapper [superseded]

The first version kept `vacate(id, messenger)` as a wrapper that only called `vacate_using(id, messenger, get_random_non_zero_id)`. The author asked why it was needed. It only saved two callers from passing the generator, which does not justify a forwarding-only function (AGENTS.md). Now `vacate` itself takes the candidate source, consistently with `new_unique_id`. Behavior and tests are unchanged.

## 2026-11-22 author review — explicit ID exclusion [accepted]

The author chose an optional excluded ID and the name `displace_actor_at`. There are only two allocation sites. Ordinary assignment passes `None`. Displacement passes `Some(id)` after removing the holder because the caller will occupy that slot. This keeps the single map removal and the original generator, with one extra comparison and no candidate callback. Both `insert` and `change_id` use the same displacement operation.

For `change_id(20, 10)`, candidate 10 matches the exclusion, 20 is still in the map, and free candidate 30 is accepted. The displaced actor queues 10 → 30 before the changing actor queues 20 → 10. The retry condition establishes exclusion for every draw. The existing public-operation tests cover actor counts, main ID and notification order. The superseded tests called the internal displacement step directly and required a candidate callback for that purpose.

Verified on Linux with uncommitted changes over `ce44b8b`. The release workspace build, workspace tests and Windows GNU bins check passed without compiler warnings. Both the headless and Vulkan clients reached `engine: ready` with loopback matchmaking settings. The headless run logged a gamepad hotplug monitor error. The Vulkan run used an NVIDIA GeForce RTX 3070 Laptop GPU.

The author confirmed completion on 2026-11-22.
