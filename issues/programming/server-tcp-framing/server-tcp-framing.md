# Frame server TCP messages and bound reads

- **Status:** Open — not implemented
- **Priority:** Normal
- **Created:** 2026-11-16
- **Stable address:** `issues/programming/server-tcp-framing/`
- **Context:** [server-tcp-framing-context.md](server-tcp-framing-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Task

The game server and matchmaking server deserialize after a single `read_buf`. TCP reads can contain only part of a message. The game-server listener also waits for input sequentially without a read timeout, letting a silent connection block later requests.

Define framing (or another explicit complete-message rule), read the complete frame and handle EOF, malformed input, size bounds and timeouts. Coordinate both ends of the server protocol and deployment compatibility.

Completion: fragmented delivery works, partial/invalid messages fail predictably and a silent peer cannot indefinitely block listener progress. Sources: `game_server/src/main.rs`, `matchmaking_server/src/main.rs`; independent of the full authoritative-server rewrite.
