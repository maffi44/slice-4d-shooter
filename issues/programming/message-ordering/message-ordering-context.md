# Context: Define and preserve message ordering

[Current task](message-ordering.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

Review point 3. Author agreed and assigned high priority (2026-11-16).

Earlier decision: keep LIFO because actor logic treats a tick as simultaneous, and `pop()` is efficient. Context: [engine refactoring](../engine-refactoring-world-isolation/), §19 round 6, §23 and §21.4. That argument does not hold for successive state assignments; retaining capacity is independent of FIFO versus LIFO. A `VecDeque` or reusable sequential buffers are candidates, not a chosen design.

Player doll timestamps already reject some old updates; that does not protect score/flag state generally. Preserve the previous decision as history, with this concrete reason for reopening it.

## Implementation log

### 2026-11-20 — design and implementation

The author asked to implement FIFO ordering and remove the possible collisions.

Cross-queue policy. The previous pump drained all broadcasts, then all direct messages, then all commands, each with `pop()`. Besides reversing successive updates, the per-kind phases reorder lifecycle commands relative to messages. Concrete cases found:

- `NetSession` turns relayed packets into commands and direct messages. `SpawnPlayersDollActor` followed by direct messages to that doll in the same tick: the messages were delivered before the spawn command ran, so they were dropped.
- `PlayerDisconnected` (`RemoveActor(peer)`) followed by a respawn of the same peer's doll (`SpawnActor`): LIFO executed the spawn first and then removed the new doll.
- `JoinTheMatch`/`NewSessionStarted` queue `RemoveAllHolesAndEffects` and then a broadcast whose handlers may queue spawns; with LIFO those spawns ran before the removal and could be removed with it.

Options: (a) FIFO inside each kind, keeping the phases; (b) one total sending order. [accepted] (b): it is the only policy under which "the order sent is the order applied" holds for state updates and lifecycle commands together, and it matches the order in which `NetSession` receives packets. (a) [rejected]: keeps the spawn/message and remove/spawn anomalies above.

Storage. `Message` is 240 bytes, `Command` 1520 bytes (measured with `size_of` on Linux x86_64), so a single `VecDeque<enum>` [rejected] would copy and store command-sized slots for every message. Accepted: per-kind `VecDeque`s plus a `VecDeque<QueuedKind>` (1 byte per item). Double buffering (swap per generation) was not needed: `VecDeque::pop_front` is O(1) and keeps capacity.

ID changes. With FIFO, the old push order (old -> new, then new -> random) would rename a reference to `old` twice. `Actors::place` was replaced by `Actors::vacate`, called before the changed actor's `change_id`, so the displaced actor's notification is queued first. `change_id` vacates while `old_id` is still in the map, so the displaced actor's random id can never be `old_id` (previously possible with negligible probability). This supersedes the constraint recorded on 2026-11-20 from main actor ID collisions; that task's statement about LIFO is historical.

Behavior changes to watch in play: commands now run as soon as their turn comes, not after all messages; a handler that sends `RemoveActor` for an actor and later sends it a direct message now loses that message. No such pattern was found in the actors (searching `send_command` / `send_direct_message`). Remaining unrelated limitation: WebRTC reliable and unreliable channels are separate, so the relative order of packets on different channels is defined only by `NetSystem` polling order, not by sending order.

Verification (Linux x86_64, uncommitted on top of `da38ae0`): `cargo check --workspace --bins`, release workspace build, `cargo test --workspace` (new tests: `messenger::tests` for cross-kind order, nested sends and outer-world FIFO; updated `actors::tests` for notification order) and Windows GNU bins check pass without warnings. Headless (12 s) and windowed (20 s) runs of the default map offline showed no errors or panics. Not verified: online play (needs local matchmaking/game servers and manual interaction), and a receiver-level test of `MoveWBonusSpot` (needs `EngineSharedSystems`).

### 2026-11-20 — single queue (author review)

The author confirmed the problem list above and asked for one queue for all items instead of per-kind buffers plus an order queue, for speed (one sequential buffer to read), and asked what makes the items large.

Measurements (`size_of`, Linux x86_64, temporary probe test, removed afterwards):

- `ActorWrapper` 1504 bytes: an enum is as large as its largest variant. Largest actors: `PlayerFor2d3dExample` 1504, `ObstacleCourseFreeMovementPlayer` 1200, `MainPlayer` 1168, `Observer` 960, `PlayerDoll` 608; frequently spawned effects are 160-240 bytes (`MachinegunShot` 208, `HoleGunShot` 240, `Wave` 240). `Command` was 1520 bytes only because `SpawnActor` stored an `ActorWrapper` inline.
- `Message` 240 bytes: `from: u128` (16, alignment 16) + `remote_sender` + `MessageType` 208. `MessageType` is dominated by `SessionControllerMessage` 208, whose `SendToPlayer(NetMessageToPlayer, ActorID, Reliability)` carries outgoing network protocol messages through the bus. `CommonActorsMessage` is 112 (`Transform` 96). Most other message enums are 0-80 bytes.

Decision. [accepted] One `VecDeque<Queued>` with `CommandType::SpawnActor(Box<ActorWrapper>)`: `Command` becomes 48 bytes, `Queued` 256 bytes (the `Direct` variant: id + message; the discriminant fits into a niche). Supersedes the per-kind storage accepted earlier today ([superseded]: its premise was that a single queue would have 1520-byte slots; boxing the spawn payload removes that premise). Cost: one heap allocation per spawn (shots, impacts, waves, dolls); constructors of most of these actors already allocate, and spawns are far rarer than messages. About 20 call sites now pass `Box::new(...)`.

Not changed: `Message` stays 240 bytes. Shrinking it would mean boxing the outgoing network variants (an allocation per sent network message, several per tick) or sending network traffic to SessionController outside the actor bus; neither is proportionate here. [open] if profiling shows message size matters.

Performance assessment (inference, not measured): with per-kind queues each pop read the order queue and one data queue, both sequential and prefetch-friendly; the gain of a single queue is mainly simpler code without a cross-queue invariant. The dominant cost of a broadcast is iterating every actor in the `HashMap` (actors of hundreds of bytes to 1.5 KB) and dispatching `recieve_message`, not the queue layout.

Verification (Linux x86_64, uncommitted on top of `da38ae0`): workspace check, release build, workspace tests (including new `queue_items_stay_message_sized`), Windows GNU bins check pass without warnings; headless (12 s) and windowed (20 s) offline runs without errors.

Reviewed by the author and closed on 2026-11-20. Follow-up research on cache-friendly layout across the engine: [cache-friendly memory layout](../cache-friendly-memory-layout/).
