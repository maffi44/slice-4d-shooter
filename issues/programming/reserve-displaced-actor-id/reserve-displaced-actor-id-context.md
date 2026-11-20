# Context for exclude the displaced actor's old ID from replacement candidates

[Current task](reserve-displaced-actor-id.md)

## 2026-11-20 code review on `4007599`

`Actors::vacate` removes the old holder before `get_new_random_uniq_id` checks the occupied keys. If the generator returns that same ID, the displaced actor is reinserted there and overwritten by the incoming actor. If it was the main actor, the camera ID then identifies the incoming actor. This is a code-level counterexample with negligible probability for a random `u128`.

Both insertion and `ChangeMyId` use this helper. During `change_id`, the changing actor's source ID is already kept occupied while the destination is vacated. Maintain that protection and the displaced-first FIFO notifications. See [actor storage](../../../src/engine/world/actors.rs) and [main actor ID collisions](../main-actor-id-collisions/).
