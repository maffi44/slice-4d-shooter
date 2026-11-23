# Close connections when their owner is removed

- **Status:** Done
- **Priority:** High
- **Created:** 2026-11-16
- **Closed:** 2026-11-23
- **Resolution:** Done
- **Stable address:** `issues/programming/session-connection-lifetime/`
- **Context:** [session-connection-lifetime-context.md](session-connection-lifetime-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Task

Removing a `SessionController` or its world used to leave a transport in the shared `NetSystem`. Incoming packets accumulated without a consumer.

Tie connection lifetime to its logical owner. Cover actor removal, root replacement, nested-world destruction, failed connection attempts and application shutdown. Choose the simplest ownership/cleanup mechanism after examining all paths; a bare `Drop` on a copyable ID cannot access shared transport by itself.

Completion means removed owners leave no active connection or orphan receive queue/task, and surviving owners retain their connections. Verify repeatedly, including while connecting. Source: `src/actor/session_controller/net_session.rs`, `src/engine/shared_systems/net.rs`, world/actor removal paths.

## Decision

[accepted] `ConnectionHandle` owns an `Arc<()>`. Moves transfer ownership and clones share a connection. The token's allocation identifies the connection in its issuing system. The registry holds a `Weak` and removes expired entries before polling on the next tick. Owners release handles by dropping them. The author superseded the earlier single-owner and public `close` designs after considering shared ownership.

[accepted] `send` and `receive` report connection errors. `receive` drains buffered packets before reporting a disconnect. `state` and `local_peer_id` remain queries. See [engine architecture](../../../docs/engine-architecture.md#networking).

## Resolution

Session transitions and world or actor destruction release handles through ordinary Rust ownership. The next network tick removes entries with no surviving owner. WebSocket tasks can be cancelled during connect, read, write and close. The WebRTC transport cancels its owned Matchbox task, and Matchbox 0.15 releases the native peers asynchronously. A WebRTC enqueue failure is returned to the caller.

Local checks covered stalled handshakes, an incoming final packet before disconnect, a blocked writer, shared and foreign handles, established WebRTC traffic and application shutdown. The UDP count returned to baseline on every measured owner removal. The local game client also rejoined after a server loss. See the [migration results and verification limits](session-connection-lifetime-context.md#2026-11-23--matchbox-015-migration).
