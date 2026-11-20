# Context: Close connections when their owner is removed

[Current task](session-connection-lifetime.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

Review point 2. The author confirmed the finding and called it very high priority on 2026-11-16. No implementation mechanism was selected.

The original design deliberately made transport application-wide and sessions per-world. Rendering already uses an owner lifetime token, but network handles currently do not express ownership. Read [refactoring context](../engine-refactoring-world-isolation/), §13 and §21.9, before changing these responsibilities. Keep transport protocol-agnostic; cleanup must also work for arbitrary nested worlds.

## Discussion log

### 2026-11-20 — analysis and proposal

Paths examined:

- `SessionController` stores a plain `Copy` `ConnectionHandle` in `SessionState`. `NetSystem::close` is called only on explicit failure/cancel paths in `net_session.rs`. Removing the actor (`Actors::remove`, `RemoveAllHolesAndEffects` does not touch it), replacing the root world or dropping a nested world drops the handle and leaves the `Connection` in `NetSystem`: it keeps being polled every tick and its receive buffer grows without a consumer. Application shutdown drops `NetSystem` with all connections, which is correct.
- Dropping a transport is enough to close it, with one gap. WebRTC: dropping `WebRtcSocket` closes its channels, and matchbox_socket 0.12 `message_loop` exits when the outgoing queue's receiver end closes (`webrtc_socket/mod.rs`, "most likely means the socket was dropped"). WebSocket: dropping the outgoing sender ends the writer task, which closes the sink. But the reader task ignores `event_sender.send` errors and lives until the remote side closes the stream, and the connect phase runs until its timeout. That is an orphan task even with today's explicit `close`.
- The render system already ties per-world GPU state to an owner token (`Weak<WorldRenderShared>`, slots swept with `strong_count() > 0`).

Options:

1. [proposed] An owning handle with a liveness token. `ConnectionHandle` (name kept) becomes non-`Copy`: `{ id: u32, owner: Arc<()> }`. `NetSystem` stores `Weak<()>` per connection. `close(handle)` consumes the handle and removes the connection immediately. A handle dropped without `close` (actor removed, world dropped, owner panicked out of a state) expires; `NetSystem::tick` removes expired connections before polling, so a dropped owner's connection is never polled again. Cost: one small allocation per `connect`, one atomic load per connection per tick. It is the same pattern as render slots, keeps `NetSystem` central (§13.3: several independent connections managed via handles; a future multiplexed UDP transport or the server engine needs a central demultiplexer), and needs no special cases for nested worlds.
2. [rejected] The actor owns the transport object and `NetSystem` becomes a factory with the runtime. `Drop` would close immediately and there would be no registry, but polling moves into every owner, and it contradicts the central-transport decision (§13.3) and future shared sockets.
3. [rejected] A `Drop` that sends the id over a channel to `NetSystem`. It needs the same handle change as option 1 plus a channel and a drain step: more machinery for the same result.

Also needed with any option: the WebSocket reader and connect tasks must stop when the consumer is gone (`tokio::select!` on `event_sender.closed()`), otherwise "no orphan task" does not hold.

Awaiting the author's decision.
