# Context: Authoritative server and transport

[Current task](network-authoritative-server-and-transport.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

## Preserved history (before the 2026-11-16 documentation migration)

The following original issue is preserved in full. Its old filename rules, status summaries and “final” architecture headings describe their original rounds and are superseded by the current task and AGENTS.md where they disagree. Section numbers remain available for older references.

# Network: authoritative server, drop emulated P2P, UDP/TCP transport

- **Created:** 2026-11-12
- **Status:** 🔵 Backlog / long-term. **Not part of the current engine refactoring.**
- **Related:** `issues/programming/engine-refactoring-world-isolation/` §13. That refactoring turns `NetSystem` into a transport abstraction with connection handles and moves session logic into actors (`SessionController`). This issue builds on that.

## Context (from the author)

- WebRTC (via Matchbox) was chosen when a browser version was being considered. The original plan was direct peer-to-peer connections.
- In practice connections often went through TURN servers. The architecture drifted into **client-server with P2P-style logic**: clients address each other (`ClientMessage::DirectMessageToPlayer` / `BoardcastMessageToPlayers`), and the game server **relays** these messages. WebRTC remained mainly for UDP-like delivery from the browser.
- A browser version is no longer a significant priority.
- Clients exchanging authoritative game data through a relay is **vulnerable to cheating** and unsuitable for a competitive shooter.

## Goals

1. **Authoritative game server:** the server becomes the source of truth for game state.
2. **Drop the emulated P2P message model.** Clients talk to the server, not to "peers".
3. **Possibly replace WebRTC with plain UDP/TCP** (or a UDP-based protocol) as the transport.
4. **Client-side networking lives in dedicated actors.** For example, `MainPlayer` sends ordinary in-world messages to `SessionController`, which owns the server connection and talks to it via `NetSystem`. The author isn't yet sure this split fits every case; it will be decided when designing this task.
5. Further independent connections (server browser, statistics) as separate actors using `NetSystem` handles.

## Notes

- Changing the transport should touch only `NetSystem` internals, thanks to the transport abstraction from the refactoring.
- Moving to an authoritative server changes the protocol (`game_server/src/client_server_protocol.rs`), the server logic (`game_server/src/main.rs`) and the client session actors. If the refactoring adopts "Variant 1" (all game traffic through `SessionController`), the client-side change is concentrated there.
- Client and server versions must match (see `AGENTS.md`, Release / CI).

## Open questions (to be detailed when this task starts)

- Server tick model, client prediction, reconciliation, lag compensation for hit detection.
- Transport choice (raw UDP + own reliability, an existing crate, QUIC, …).
- Matchmaking flow changes.
- Migration and compatibility strategy for deployed servers.

## Discussion

1. **2026-11-12. Created from round 3 of the refactoring discussion.** The author explained the WebRTC/P2P history and the long-term goals; Claude analyzed `net.rs` and recommended "Variant 1".
2. **2026-11-12. Round 4.** The author approved Variant 1 for the refactoring: all game traffic goes through `SessionController`, and `NetSystem` becomes a transport with connection handles. So this issue's client-side work will be concentrated in `SessionController`.
