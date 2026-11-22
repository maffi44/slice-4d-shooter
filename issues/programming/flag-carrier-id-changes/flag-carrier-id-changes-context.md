# Context for keep the captured flag attached when its carrier changes ID

[Current task](flag-carrier-id-changes.md)

## 2026-11-20 code review on `4007599`

`FlagStatus::Captured(id)` drives the direct `GiveMeTargetPosition` sent each tick. The flag's common-message handler handles transform updates and disconnects but falls through on `IWasChangedMyId`. When the carrier changes ID, the request can be dropped or delivered to a different actor that acquired the old ID. The same stored ID is compared with the sender of `PlayerDied`. This was traced in code during review.

`MoveWBonusSpot` already rewrites a stored ID when the notification's `from` matches. On collision, the displaced actor's notification is queued first. The actor-container tests verify changes in the table and emitted notifications, without exercising the flag. See [flag](../../../src/actor/flag.rs), [bonus spot](../../../src/actor/move_w_bonus.rs) and [message ordering](../message-ordering/).

## 2026-11-22 implementation

The handler matches the notification's `from` against the stored carrier, the same rule `MoveWBonusSpot` uses. Collisions are correct because notifications are processed FIFO and the displaced actor's notification is queued first ([message ordering](../message-ordering/), [displaced ID](../reserve-displaced-actor-id/)). Carrier A (10) takes 20 from B: `(20 -> R)` does not match, then `(10 -> 20)` updates to 20. Carrier B (20) is displaced: `(20 -> R)` updates to R, then `(10 -> 20)` no longer matches. In both cases the flag follows the original carrier.

Updating the field directly instead of calling `set_flag_captured_status` avoids replaying the capture wave and sound. As the task requires, the server is not notified. [open] Observation, not checked in play: `client_server_protocol::FlagStatus::Captured` also carries an `ActorID`. A later server status (for example `JoinedToSession` or `SetFlagStatus`) would carry the ID the server knows. This task does not change how server and local IDs are reconciled.

To test without `EngineSharedSystems`, the tick's position request and the `PlayerDied` check became methods that take `Messenger`. Tests generate real notifications with `Actors::change_id` and deliver them to the flag in queue order. They check the new carrier ID, the next `GiveMeTargetPosition` recipient, and that only the new ID's death produces `DropedFlag`. A new `#[cfg(test)] Messenger::queued_direct_messages` exposes direct messages to tests. `queued_broadcast_messages` became `pub(crate)` for actor tests. Online play with a real rename was not run (a rename cannot be triggered reliably in play).

Verification on uncommitted changes over `69d51e4` (Linux x86_64): `cargo build --release --workspace --locked` and `cargo test --workspace --locked` pass without compiler warnings, and so does `cargo check --target x86_64-pc-windows-gnu --bins --locked`. With loopback matchmaking settings, the headless client (15 s) and the Vulkan client (20 s) on the default map reached `engine: ready` with no panics.

## 2026-11-22 review and closure [accepted]

A join snapshot can already say peer 20 carries the flag while a map actor still holds local ID 20. When the remote doll spawns, that map actor announces 20 → R. Treating the snapshot as an established local binding made the flag follow the map actor. This supersedes the original assumption that the local carrier always exists at capture time.

Keep the server peer ID in the capture status and bind its local ID only when the player identifies itself. One lookup per new capture finds an existing player, including a doll already displaced from its peer ID. A later spawn announces availability in the same world. While waiting, the flag sends no position requests. Once bound, the normal direct request and FIFO rename paths apply. A local player's death carries its peer ID so an already queued death can be recognized before the first binding reply. These messages stay inside the world.

A regression test runs the real messenger with a join snapshot, an occupied slot and a late doll, checks the returned position, then renames the doll and captures again under the original server peer ID.

Verification on the working tree over `69d51e4`: the Linux release workspace build, workspace tests and Windows GNU bins check passed. The no-updater client reached `engine: ready` in both headless and Vulkan with loopback matchmaking settings. Headless logged a gamepad hotplug monitor error. The regression ran with an in-process actor world and the real message queue. No multiplayer server was contacted.

Closed at the author’s request after review.
