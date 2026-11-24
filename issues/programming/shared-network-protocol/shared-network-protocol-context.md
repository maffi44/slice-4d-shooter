# Context for the shared network protocol

[Current task](shared-network-protocol.md)

## 2026-11-24 — First pass on baa9800

The author requested a separate task if extraction has a distinct outcome. The broader [dependency audit](../dependency-and-build-cleanup/) can clean unused dependencies independently. Axum is used by the game server even after extraction.

Linux x86_64 and Windows GNU locked normal dependency trees show `slice-4d-shooter → game_server → matchbox_signaling → axum 0.8.9`. The client also declares `matchmaking_server` directly, and reaches it through `game_server`. Cargo attaches ordinary dependencies to the package even when a consumer imports only its library target. This establishes compilation reachability. Release executable size needs a before/after measurement with identical build settings.

| Exchange | Current owner | Transport owner after the proposed move |
|---|---|---|
| Client ↔ game server, e.g. join or gameplay events | `game_server/src/client_server_protocol.rs` | Client and game server keep Matchbox/WebRTC |
| Client ↔ matchmaking, e.g. request version and return an address | `matchmaking_server/src/matchmaking_server_protocol.rs` | Client and matchmaking keep WebSocket |
| Game server ↔ matchmaking, e.g. query player count | Same matchmaking file | Both servers keep their TCP sockets |

[proposed] Example consumer import: `use net_protocol::gameplay::ServerMessage;`. The client still decodes a received Matchbox packet via `ServerMessage::from_packet(packet)`. In locked Matchbox 0.15, `Packet` is exactly `bytes::Bytes`. Name that same type in the shared library. Keep the existing conversion from the serialized vector and the same ownership at send/receive. The matchmaking helpers can continue returning `Vec<u8>`. A crate that imported `matchbox_socket::Packet` would also pull native RTC into an otherwise lightweight matchmaking build. An extra adapter per transport is unnecessary for these signatures.

One crate with two modules accommodates the two different `ClientMessage` enums and keeps a single wire definition per type. Two independent protocol packages would also isolate the servers but add a second manifest for little saving once transport dependencies are removed. The shared library should need only Alkahest with its existing derives and `bytes` as ordinary direct dependencies. Applications that call Alkahest directly still declare it. The matchmaking binary currently compiles its protocol file again through `pub mod matchmaking_server_protocol` and has a `crate::matchmaking_server_protocol` import. Migrate these along with the exported server library targets.

[open] Cargo feature unification hides dependencies. For example, the production client constructs Tokio's multi-thread runtime but only its dev dependencies explicitly request `rt-multi-thread`. Signaling also enables that feature. Declare the features actually used by each binary and check each package alone without dev targets. Matchmaking uses Tokio TCP and buffered async I/O, and the game server uses a TCP listener and async reads/writes. Preserve `-p game_server` and `-p matchmaking_server` as used in deployment.

[proposed] Capture representative baseline packets before moving the types, then exercise old encode → new decode and the reverse. Keep intentional wire spellings, numeric types, enum/field order and the TCP framing contract. A successful round-trip using only the newly moved types would miss a symmetric wire-format change.

## 2026-11-24 — Author approval and compatibility baseline

[accepted] Use the third option discussed with the author. Encoding owns a `Vec<u8>`, decoding borrows `&[u8]`, and callers convert the vector at a transport boundary that needs a different owner. The Matchbox `Bytes` alias proposed above is superseded. A slice carries the pointer and length and is sufficient for decoding. The author preferred the standard-library contract and accepted an explicit `.into()` on send. Keep the existing gameplay failure behavior and matchmaking decode errors.

Captured six packets with the pre-move locked release libraries at `baa9800`: a flag request, a new-session reply, a versioned matchmaking request, a loopback address reply and both directions of the server player-count exchange.

[accepted] Matchmaking decoding now lives alongside encoding in its two existing message types. Each `from_packet(&[u8])` preserves the original Alkahest result, so callers retain their malformed-packet handling without importing Alkahest. Gameplay decoding still returns `Option<Self>`. The three application manifests depend directly on the protocol and only the protocol declares Alkahest.

### Verification on the uncommitted Linux tree based on baa9800

Linked one probe against the baseline release libraries and another against the extracted library. Each wrote six files, checked the other implementation's files byte-for-byte and decoded their expected field values. Both directions passed for gameplay flag request/session reply, matchmaking version request/address reply and TCP status query/player-count reply. Three integration tests retain representative baseline bytes under `net_protocol/tests/`.

Linux and Windows GNU isolated graphs satisfy the package boundaries. The client keeps native Matchbox socket, while signaling and Axum are reachable only from the game server among the three apps. The isolated protocol graph contains eight package/version pairs including itself and the macro dependencies. Linux per-app checks, the release workspace build and the Windows GNU workspace bins check passed. The sandbox denied binding a loopback socket, preventing live WebSocket/TCP/WebRTC verification. The [dependency audit](../dependency-and-build-cleanup/) records the full workspace test and runtime results.

### Debug codec stack use

The fixed-byte gameplay test exposed a pre-existing stack overflow. On Linux x86_64 the old and extracted unoptimized `ClientMessage::to_packet` machine code both reserve at least `0x3a5000` bytes in their prologue (about 3.64 MiB). The debug test failed with 2 and 3 MiB stacks and passed with 4 MiB. Disabling the Alkahest `inline-more` feature in an isolated build also reproduced the overflow. The locked Alkahest derives and serializer contain unconditional inlining.

Enable optimization level 1 for the protocol package alone in the dev profile, inherited by tests. This also fixes the same path for actual debug callers on ordinary worker stacks. Increasing just the test stack would leave that call unsafe in debug gameplay. Release settings and the packet layout stay the same.

All three retained fixtures passed with the default test-thread stack after the profile change. In the isolated level-1 build the same encoder prologue reserved 440 bytes plus saved registers, compared with the multi-megabyte unoptimized frame. This is a measurement of that function frame on this target, not of the entire call chain.

[accepted] 2026-11-24 — The author reviewed the result and confirmed closure.
