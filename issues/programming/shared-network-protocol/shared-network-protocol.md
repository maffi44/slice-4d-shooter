# Extract the shared network protocol into a workspace library

- **Status:** Done — accepted by the author
- **Priority:** Normal
- **Created:** 2026-11-24
- **Closed:** 2026-11-24
- **Resolution:** Done
- **Stable address:** `issues/programming/shared-network-protocol/`
- **Context:** [Design, compatibility baseline and verification](shared-network-protocol-context.md).

## Scope and result

[accepted] The client and both servers now depend directly on `net_protocol`. Its `gameplay` and `matchmaking` modules own message definitions and Alkahest codecs. The three applications own sockets, tasks and connection policy. The game server continues hosting signaling. Existing package/binary names and hosting commands are preserved.

[accepted] Encode to an owned `Vec<u8>` and decode from `&[u8]`. The transport caller passes `message.to_packet().into()` when it needs Matchbox's packet type. The protocol library's only direct dependency is Alkahest. See the [engine architecture](../../../docs/engine-architecture.md#networking) for the caller contract.

## Completion and verification

The isolated Linux and Windows GNU client trees exclude both server packages, signaling and Axum. The matchmaking server excludes Matchbox socket/signaling. Old and new binaries of a codec probe exchanged six byte-identical fixtures covering the three application exchanges in both directions. Regression tests retain one baseline packet per exchange. A live session with local servers remains unverified because the sandbox denied socket creation. See the context for exact checks.

General dependency and feature cleanup is tracked in [the dependency audit](../dependency-and-build-cleanup/). Changes to the wire protocol or [TCP framing](../server-tcp-framing/) are separate work.
