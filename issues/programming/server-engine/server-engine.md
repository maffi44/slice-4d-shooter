# Server side of the engine: many cheap session instances

- **Status:** Open — design pending
- **Priority:** Long-term
- **Complexity:** Very complex
- **Created:** 2026-11-20
- **Stable address:** `issues/programming/server-engine/`

## Goal

Development so far has focused on the client side of the engine. The game also needs a server side of the engine that runs game sessions headless, with these mandatory targets (author, 2026-11-20):

- very high performance and minimal per-session cost, so that as many session instances as possible run on weak, cheap servers;
- minimal multithreading: a session must not create its own threads; every thread and runtime in the headless path must be accounted for.

## Current state (evidence, 2026-11-20)

- `game_server/` is a separate relay process per session: `matchmaking_server` starts `./game_server` with `std::process::Command` and kills it with `kill`. It uses a current-thread Tokio runtime. It does not run the engine or world simulation; gameplay is client-authoritative.
- The engine's headless mode has no window, GPU, rendering or sound device, but it is not thread-minimal: `engine/loader.rs` and `shared_systems/resources.rs` each build a multi-thread Tokio runtime, and loading uses `spawn_blocking` (`resources.rs`, `world/world_descriptor.rs`). The render thread (`render_scheduler.rs`) is not created in headless mode; this needs confirming together with the audio, input and settings systems.

## Initial idea [proposed]

A shared host ("combine") process that runs many sessions. Each session is a lightweight unit: a thread or, preferably, a task on a shared scheduler. Separate processes are also possible but cost more. Services that sessions can share live once per host: one powerful Tokio runtime for networking and resource loading (tens of thousands of tasks), shared immutable resources such as parsed maps, and shared timers. The per-session part keeps only its world state and simulation tick. This is the author's starting idea and a preference, not an approved design.

## Open questions [open]

- Session isolation unit: task on a shared runtime, a fixed pool of worker threads each ticking many sessions, a thread per session, or a process per session. Trade-offs: cost per session, fault isolation (a panic or stall in one session), tick-timing accuracy.
- Tick scheduling for many sessions on few threads without accumulated lateness ([tick scheduling](../engine-tick-rate/)).
- Which shared systems a server session needs (`EngineSharedSystems` without input, audio, UI and render), and how per-world state stays separate from host-wide services.
- Relation to [authoritative server and transport](../network-authoritative-server-and-transport/): server-side simulation is the base for authority; transport choice and the number of connections per host affect the design.
- Interaction with matchmaking: starting and stopping sessions inside the host instead of spawning processes; deployment and protocol compatibility with deployed clients.
- Memory per session and cache behaviour ([cache-friendly memory layout](../cache-friendly-memory-layout/)).
- How to measure: sessions per core and memory per session on a reference weak server; a load test with simulated clients against local servers only.

## Completion criteria

An agreed design with a usage example (how the host starts, ticks and stops a session; which services are shared), a thread and runtime audit of the headless path, and measured per-session cost. Implementation proceeds in stages after the design is approved.
