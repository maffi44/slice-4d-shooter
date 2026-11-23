# Context: Close connections when their owner is removed

[Current task](session-connection-lifetime.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

Review point 2. The author confirmed the finding and called it very high priority on 2026-11-16. The mechanism (option 1 below) was approved on 2026-11-23.

The original design deliberately made transport application-wide and sessions per-world. Rendering already uses an owner lifetime token, but network handles currently do not express ownership. Read [refactoring context](../engine-refactoring-world-isolation/), §13 and §21.9, before changing these responsibilities. Keep transport protocol-agnostic; cleanup must also work for arbitrary nested worlds.

## Discussion log

### 2026-11-20 — analysis and proposal

Paths examined:

- `SessionController` stores a plain `Copy` `ConnectionHandle` in `SessionState`. `NetSystem::close` is called only on explicit failure/cancel paths in `net_session.rs`. Removing the actor (`Actors::remove`, `RemoveAllHolesAndEffects` does not touch it), replacing the root world or dropping a nested world drops the handle and leaves the `Connection` in `NetSystem`: it keeps being polled every tick and its receive buffer grows without a consumer. Application shutdown drops `NetSystem` with all connections, which is correct.
- Dropping a transport is enough to close it, with one gap. WebRTC: dropping `WebRtcSocket` closes its channels, and matchbox_socket 0.12 `message_loop` exits when the outgoing queue's receiver end closes (`webrtc_socket/mod.rs`, "most likely means the socket was dropped"). WebSocket: dropping the outgoing sender ends the writer task, which closes the sink. But the reader task ignores `event_sender.send` errors and lives until the remote side closes the stream, and the connect phase runs until its timeout. That is an orphan task even with today's explicit `close`.
- The render system already ties per-world GPU state to an owner token (`Weak<WorldRenderShared>`, slots swept with `strong_count() > 0`).

Options:

1. [accepted] An owning handle with a liveness token. `ConnectionHandle` (name kept) becomes non-`Copy`: `{ id: u32, owner: Arc<()> }`. `NetSystem` stores `Weak<()>` per connection. `close(handle)` consumes the handle and removes the connection immediately. A handle dropped without `close` (actor removed, world dropped, owner panicked out of a state) expires; `NetSystem::tick` removes expired connections before polling, so a dropped owner's connection is never polled again. Cost: one small allocation per `connect`, one atomic load per connection per tick. It is the same pattern as render slots, keeps `NetSystem` central (§13.3: several independent connections managed via handles; a future multiplexed UDP transport or the server engine needs a central demultiplexer), and needs no special cases for nested worlds.
2. [rejected] The actor owns the transport object and `NetSystem` becomes a factory with the runtime. `Drop` would close immediately and there would be no registry, but polling moves into every owner, and it contradicts the central-transport decision (§13.3) and future shared sockets.
3. [rejected] A `Drop` that sends the id over a channel to `NetSystem`. It needs the same handle change as option 1 plus a channel and a drain step: more machinery for the same result.

Also needed with any option: the WebSocket reader and connect tasks must stop when the consumer is gone (`tokio::select!` on `event_sender.closed()`), otherwise "no orphan task" does not hold.

Awaiting the author's decision.

### 2026-11-23 — decision and implementation

The author approved option 1, removal at the start of the next `NetSystem::tick` (rather than instantly), and keeping the explicit `close`.

Implementation (`src/engine/shared_systems/net.rs`, uncommitted on `dev`):

- `ConnectionHandle { id: u32, owner: Arc<()> }`, not `Clone`; connections are keyed by the private `id`. `Arc` rather than `Rc` so that the handle stored inside an actor does not constrain `Send`; the cost difference is irrelevant (one allocation per `connect`).
- `state`, `send`, `receive` and `local_peer_id` borrow the handle; `close` consumes it. `tick` runs `retain(owner.strong_count() > 0)` before polling and logs each expired connection at `debug` level.
- WebSocket connect phase and reader loop are wrapped in `tokio::select!` with `event_sender.closed()`. `closed()` completes when the receiver (inside `Transport`) is dropped, regardless of the writer task's sender clone. The reader ending drops its stream half; the writer ends because the outgoing sender is dropped together with the transport; then the TCP socket is closed.
- WebRTC needs no change. Checked in matchbox_socket 0.12 sources: dropping `WebRtcSocket` closes `peer_messages_out` receivers' senders, `message_loop` breaks on `Some((_, None)) | None`, and `run_socket` returns through its `select!`, dropping the signaling future, also while the signaller is still connecting.
- `net_session.rs` only changed to borrow handles. Its removal paths need no code: dropping `SessionState` drops the handle.
- `Cargo.toml`: tokio features `time` and `macros` are now declared (previously enabled only through feature unification with other crates); `[dev-dependencies]` tokio adds `net`, `io-util`, `rt-multi-thread` for the tests. `Cargo.lock` is unchanged.

Evidence that the WebSocket orphan existed even with explicit `close`: the new test `explicitly_closed_websocket_connection_ends_its_tasks` fails when the `closed()` arms are replaced by `pending()`: the server, which never answers the Close frame, sees no TCP EOF within 5 s because the reader still holds its half of the split stream. Without the sweep in `tick`, all four drop tests fail; with both in place all five pass.

Tests (`net::tests`, local sockets on `127.0.0.1:0`, no production servers):

1. `dropped_websocket_connection_closes_while_connecting`: TCP accepted, no handshake reply, 60 s connect timeout; after drop and `tick` the listener sees EOF. Three iterations.
2. `dropped_websocket_connection_ends_its_tasks`: hand-made handshake, server never answers Close; after drop the TCP connection is fully closed. Three iterations.
3. `explicitly_closed_websocket_connection_ends_its_tasks`: same with `close`.
4. `surviving_owner_keeps_its_connection`: two connections, one dropped; the other stays `Connected` and receives a server frame.
5. `dropped_webrtc_connection_closes_signaling_while_connecting`: the listener in place of the signaling server sees EOF after drop. Three iterations.

Verification on 2026-11-23, Linux x86_64, uncommitted changes on top of `68f466f`:

- `cargo build --release --workspace --locked`, `cargo test --workspace --locked` (all passed), `cargo check --target x86_64-pc-windows-gnu --bins --locked`: no warnings.
- Windowed `game-client-without-autoupdate` against local `matchmaking_server` and `game_server` (copies and configs in a temporary directory, ICE list empty): `G` connected through matchmaking (WebSocket, explicit `close`) and WebRTC; killing `game_server` produced the lost-connection HUD; after the failure timer `G` reconnected to a newly spawned game server; closing the window while connected exited cleanly and the server logged the player's disconnect.
- Headless run for 10 s: no errors.
- Limitations: actor removal, root replacement and nested-world destruction were not triggered in game (no input path reaches them reliably); they all reduce to dropping the handle, which the unit tests cover at the `NetSystem` level. A connected WebRTC session dropped without `close` was verified only through the WebRTC connecting-phase test and the reasoning about matchbox above.


### 2026-11-23 — review: operations return `Result`

The author reviewed the implementation and found no visible problems, but objected conceptually to silent fallbacks: `receive` on an unknown handle returned an empty list, `send` silently did nothing. Author's proposal: every operation through a handle returns a `Result` whose error covers the possible connection errors; `state` may stay a plain value, since it is itself the description of the connection's state. The author left the details open and asked to check whether it is realistic and to implement it if it is.

Assessment: no fundamental obstacle. Owning handles make most of the old "unknown handle" cases impossible, so the errors that remain are real connection states. Decisions taken while implementing:

- [accepted] One error type `net::ConnectionError { Connecting, Disconnected(DisconnectReason) }` for `send` and `receive`. `Connecting` is an error for both: nothing can be sent or received before the connection is established, and the caller's match reads naturally (`Err(Connecting) => wait`). The alternative `receive -> Ok(empty)` while connecting [rejected] would bring back an indistinguishable empty result.
- [accepted] `receive` returns packets that arrived before a disconnect first and `Err(Disconnected)` only once they are taken (data before end-of-stream, as in a socket read). Evidence: the matchmaking server sends `WrongGameVersionCorrectIs` and returns immediately, closing the connection; when the packet and the close are polled in the same tick, the old caller code checked `state` first, saw `Disconnected` and reported "server not found" instead of "wrong version". Returning the error as soon as the state is `Disconnected` [rejected] would have kept that loss. For WebRTC, `poll_connection` now drains the data channels in the tick that detects the disconnect (previously it returned before reading them).
- [accepted] `send` returns `Ok` when the packet is handed to the transport (not delivery). Failures the transport detects during the call (WebSocket writer stopped, WebRTC channel closed) are returned at once; `state` reports them from the next tick, because changing the state inside `send` would stop polling before queued packets are taken.
- [accepted] Queries stay plain values: `state -> ConnectionState`, `local_peer_id -> Option<u128>`. A WebSocket connection never has a peer id; that is a property of the transport, not a connection error.
- [accepted] `DisconnectReason::UnknownConnection` stays, documented as a handle of another `NetSystem` (several engines per process are planned for the server side). A panic [rejected] would be a heavier contract change for a case that is a programming error but harmless to report.
- The private session-level `ConnectionError` in `net_session.rs` keeps its name; the transport error is imported there as `NetConnectionError`.
- `net_session.rs`: the matchmaking state sends the request once `state` is `Connected` (the packet is not built while connecting) and decides by `receive`; the connected state processes buffered packets and handles the loss after the `match`, because a `Drain` in the `match` scrutinee keeps `shared.net` borrowed in the error arm. Fire-and-forget sends from actors ignore the result explicitly; the loss is handled in `tick` through `receive`.

New tests: `operations_report_connecting`, `packets_received_before_disconnect_come_before_the_error` (server sends a frame and closes the TCP connection; the packet comes first, then `Disconnected` from both `receive` and `send`).

Verification on 2026-11-23, Linux x86_64, uncommitted on top of `68f466f`: release workspace build, workspace tests (7 `net` tests among them) and Windows GNU bins check without warnings. Windowed client against local servers: with the server version set to `0.0.1` all three attempts reported `WrongVersion` and the HUD showed "connection failed: wrong version"; with the correct version (reloaded by the server) `G` connected, killing `game_server` logged `connection to the game server is lost: Disconnected(ClosedByRemote)`, `G` reconnected after the failure timer, and closing the window while connected exited cleanly with the server logging the disconnect. Headless 10 s without errors.

### 2026-11-23 — independent review of the uncommitted changes

- [open] Established native WebRTC connections retain UDP sockets after owner removal. On Linux x86_64 at `68f466f` with the uncommitted implementation, a local Matchbox signaling server and two `NetSystem` connections exchanged a reliable packet. Both handles were dropped, then `net.tick()` ran. Across three successive pairs, process-owned UDP sockets counted via `/proc/self/fd` and `/proc/self/net/udp{,6}` went from 0 at baseline to 4, 8 and 12 after cleanup. All 12 were still open 35 seconds after stopping the signaling server, with the runtime alive and no external ICE servers configured. Matchbox logged the outgoing queue closures. Ending its outer future therefore does not establish that the native peer transports have shut down. The earlier test drops the socket before a data channel exists.
- [open] The WebSocket writer can remain suspended in `sink.send().await` if its peer stops reading. Dropping the outgoing sender leaves queued packets in the receiver and does not cancel a pending write or close. The current teardown tests use an idle writer.
- [open] The WebRTC send path checks `any_channel_closed()` then calls Matchbox's panicking `send`. The socket future can exit between the two operations. Matchbox 0.12 provides `try_send` to return this failure to the caller.
- [open] Every `NetSystem` starts numbering connections at 1. Lookups and explicit close compare only this number, so a foreign handle with a matching id can send to, read from, or close a live local connection. The liveness token already distinguishes those owners.

The workspace test suite passed during review, including the seven transport tests in each client binary. The last three findings above follow from code inspection.

### 2026-11-23 — transport and handle decisions, native WebRTC close experiment

[accepted] Keep WebSocket for matchmaking. The client sends a request for a compatible game server and consumes one answer. HTTP would fit this exchange, but would require a new endpoint and compatibility with shipped clients. Plain TCP would also need message framing. Matchbox separately uses WebSocket for signaling and sends gameplay packets via WebRTC. Neither replacement would release the observed UDP sockets. Make one cancellable WebSocket task own the handshake and both halves of the connected stream. Keep reading and writing concurrent while connected. Dropping the event receiver must cancel the entire task, including a pending send or close and any queued outbound data. A regression check should stop the owner while a write is blocked and verify writer termination while the peer is still silent.

[accepted] Keep per-system polling and a unique owning handle. Remove the independent `u32` counter. Use the allocation identity of the existing `Arc<()>` as the private hash-map key (its address, used only for equality and hashing). The registry's `Weak` retains the allocation until removal, which prevents address reuse while an old entry exists. A live handle also keeps its identity if its system is destroyed. Derive the key internally for every operation, including queries and explicit close. Lookups remain expected constant time and require no new allocation or global counter. A global numeric id would add another identity beside the token. A vector scanned with pointer equality would make every send proportional to the number of connections in the system. Moving the transport into each actor would move polling out of the shared system.

Caller example: `a.send(&h, packet, reliability)` uses the connection issued by `a`. If `b.close(h)` receives that handle, it returns it in the error (`Result<(), ConnectionHandle>`). The caller can then close it through `a`. Both connections survive the mistaken call while the caller retains the returned handle. Dropping that handle still deliberately expires its own connection. Successful explicit close removes the registry entry immediately and starts background transport shutdown.

[superseded] The earlier inference from Matchbox's outer message loop covered the signaling handshake only. In the locked native stack (`matchbox_socket` 0.12.0, `webrtc` and `webrtc-ice` 0.13.0), dropping a socket ends Matchbox's message loop. Its public `WebRtcSocket::close` also only closes the application channels. Neither path awaits native `RTCPeerConnection::close`. In ICE, `AgentInternal::start_candidate` spawns a receive loop holding strong references to the candidate, underlying connection and agent. Explicit peer close stops the ICE transport and gatherer, closes the agent's candidates, and wakes those receive loops. [Locked Matchbox implementation](https://github.com/johanhelsing/matchbox/blob/2cfee6da29c97d7ae1db1cf196e418821a0736d4/matchbox_socket/src/webrtc_socket/native.rs), [locked WebRTC shutdown](https://github.com/webrtc-rs/webrtc/blob/d35569938c9de1bdd5d08f2234f90de81b4540fc/webrtc/src/peer_connection/mod.rs#L1927).

Linux x86_64, uncommitted `68f466f` tree. Two fresh processes ran the same temporary executable against an in-process signaling server with an empty ICE server list. Each established three successive pairs, exchanged a reliable packet on each pair, dropped both handles and ticked the live system. Both modes kept a reference to each native peer through the handshake metadata. Only the experimental mode scheduled and awaited native peer close on metadata drop. Counts below are process-owned UDP descriptors sampled one second after each pair was removed.

| Mode | Before connecting | After pair 1 | After pair 2 | After pair 3 |
|---|---:|---:|---:|---:|
| Drop only | 0 | 4 | 8 | 12 |
| Explicit native close | 0 | 0 | 0 | 0 |

All six explicit close calls returned success. Teardown also logged lower-layer warnings about already closed channels and transports. Counts remained 12 and 0 respectively four seconds after stopping signaling. This isolates the missing native close for established sessions. It does not measure all background tasks or exercise partial WebRTC handshakes and runtime shutdown.

[proposed] Put cleanup where the native peer is created, covering both handshake roles and cancellation during negotiation. Keep the simulation tick nonblocking and allow shutdown work to finish while its runtime is alive. The released Matchbox v0.15.0 wraps the native peer at construction and schedules asynchronous close on drop. Prefer testing that release with the client, game server and modified signaling server before maintaining a backport. The release changes the native WebRTC stack and its packet type from boxed slices to bytes. Compatibility with our existing deployment remains unverified. The experiment used the locked versions and temporary sources only. [Upstream native peer guard](https://github.com/johanhelsing/matchbox/blob/v0.15.0/matchbox_socket/src/webrtc_socket/native.rs#L400-L424).

### 2026-11-23 — shared handles and last-owner shutdown

[accepted] The author confirmed the existing allocation as the only identity and chose shared ownership. A move transfers the handle, a clone shares the connection, and dropping the last handle expires the registry entry on the next tick. A public close would let one owner invalidate another's live handle. This supersedes the single-owner decision and the proposed close that returned a foreign handle. The internal native peer close required by the transport has a separate purpose.

The private token is created once and never replaced or passed to `Arc::make_mut`. Moving or cloning the handle preserves the allocation address. The registry holds a `Weak` until removal, so the address cannot be reused while the entry exists. Pinning adds nothing to that lifetime. Session transitions release their handles by leaving scope. Clones use the same received-packet queue and the first drain consumes the packets for all owners.

UDP attribution on Linux x86_64: a standalone process linked the unmodified, locked `matchbox_socket` 0.12.0 and the project's loopback signaling server. It used no engine, registry, actor or game protocol. For each of three pairs it created two `WebRtcSocket` values with empty ICE server lists, exchanged and verified one reliable payload, dropped both sockets and awaited both Matchbox futures with a five-second deadline. All six futures returned `Ok(())`. One second after each pair the process held 4, 8 and 12 UDP descriptors, starting at zero. A second fresh process called Matchbox's public `WebRtcSocket::close()` on both sockets before dropping them. The counts and future results were identical. Both processes still held 12 UDP sockets two seconds after stopping signaling.

Combined with the earlier experiment that closed each underlying native peer and returned to zero, this identifies the ownership boundary. Native ICE creates the candidate sockets during WebRTC negotiation. Matchbox 0.12 discards its peer state without calling native peer close, leaving ICE receive tasks holding the sockets. The application can release every handle and await every Matchbox future and still observe this behavior. Four descriptors per pair was the count on this machine, not a fixed cost per session. The project still needs a transport integration that releases them.

Verification on Linux x86_64 with the uncommitted changes over `68f466f`: release workspace build, workspace tests (32 per client binary, including last-owner and foreign-system checks), and Windows GNU bins check passed without compiler warnings. The release headless client loaded its root world and ran until the seven-second smoke-test timeout. The Vulkan client joined an isolated local game server through local matchmaking and exited normally when its window closed. Both runs used a loopback matchmaking override and empty external ICE server lists.

### 2026-11-23 — Matchbox 0.15 migration

[accepted] The author approved upgrading the Matchbox components and closing this task once verification passes. The client and game server use `matchbox_socket`. The game server embeds the local `matchbox_signaling` library. There is no standalone `matchbox_server` executable in the deployment. Rebased the vendored crate on the published 0.15.0 source after comparing local Rust files with the published 0.12.0 source. Only the client/server inactivity check was project-specific. Its actual first check was after 15 seconds, followed by five-second checks for more than five seconds of silence. Preserve that behavior for host and clients and correct the old three-second description in NOTICE. Kept both original licenses.

Both endpoints now use Matchbox's immutable byte buffer directly for the existing alkahest payloads. Client enqueue uses the fallible channel operation. The matchmaking connection still uses WebSocket. Connect, read, write and close now belong to one cancellable task so owner removal also cancels a blocked send.

The existing cancellation test exposed a change in the 0.15 startup sequence. `run_socket` waits for `new_signaller` before polling the outgoing queues. With a local TCP peer that accepted the socket but withheld the WebSocket handshake, dropping the handle alone left the signaling connection open past the five-second test deadline. The WebRTC registry entry now retains the Tokio socket task and aborts it on drop. The native guard then owns asynchronous peer shutdown whenever negotiation has created a peer. The stalled-signaling test passes with this change.

Repeated the UDP measurements using the actual engine transport and 0.15.0 registry artifacts on Linux x86_64. Signaling ran locally with no external ICE services. Three connected pairs each exchanged verified payloads on both reliable and unreliable channels. There were six process-owned UDP descriptors per active pair on this host and zero after last-owner drop and a tick in all three rounds. In three further rounds, a passive signaling peer received an offer and withheld its answer. The engine transport still reported Connecting and had live UDP descriptors. Dropping its handle released all of them while the passive peer stayed connected. Dropping the entire registry with a live pair and immediately shutting down Tokio also returned the count from six to zero. A separate WebSocket regression holds a writer under backpressure and verifies that its queued payload is released while the remote side is still silent.

Release workspace build, workspace tests (33 per client binary and the vendored signaling tests), and the Windows GNU workspace bins check passed on the uncommitted tree over `68f466f`. For compatibility, the Linux Vulkan client joined the local game server with both endpoints on 0.15, with a 0.12 client and 0.15 server, and with a 0.15 client and 0.12 server. Each window exited normally and the server observed the disconnect. A first GUI automation attempt sent the connect key before input was accepted. The three reported runs waited for readiness and focused the test window. Headless loaded its world and ran until the eight-second timeout. Sandbox udev access was restricted on the initial headless run; the repeat on the host reached readiness without that error. Windows runtime and public NAT/TURN paths were outside these local runs.

A silent signaling client was removed at 15.00 seconds while its active host continued sending keepalives and received PeerLeft. With both game endpoints on 0.15, terminating the local game server produced the lost-connection state after about 29 seconds and the Vulkan client successfully rejoined a new server. The local compatibility checks used no production services. Actor removal and nested-world destruction use the ordinary handle-drop path; the socket measurements exercised that path directly, outside the gameplay UI. The author requested closure conditional on the migration and final checks succeeding. Closed as Done on 2026-11-23.
