# Engine architecture reference

Implementation reference updated 2026-11-24. For working rules see [AGENTS.md](../AGENTS.md); for task status see the [open tasks](../issues/open-tasks.md).

## Engine architecture (`src/engine.rs`, `src/engine/main_loop.rs`)

The design, its rationale and history are in [engine-refactoring-world-isolation](../issues/programming/engine-refactoring-world-isolation/) (context Part I: rationale, Part III: historical target architecture; the main task tracks current status). Summary:

**`Engine`** owns the root world, the shared systems, a private `Arc<ResourceSystem>` for cache maintenance and the optional window. It contains no loading state or startup handles. The engine knows only the root world. Nested worlds are owned and simulated by actors.

Startup is separate from simulation ([decision and implementation context](../issues/programming/nonblocking-engine-startup/)):

```rust
let loading = engine::load(start_level, args.graphic_backend, args.headless);
MainLoop::run(loading)?;
```

- `engine::load` returns `EngineLoader`. It starts async map/settings loading and GPU instance discovery concurrently; a dedicated loading thread prepares shared systems (including audio/input/UI), coordinates dependencies and owns intermediate handles. The Tokio runtime ultimately moves into the ready shared systems.
- Graphical `MainLoop` owns the winit application. First `resumed` creates the window (duplicate resumes are guarded). The loader requests a surface through its result channel; only window/surface creation happens in the event-loop callback, preserving Metal's main-thread requirement. Adapter/device, shared GPU resources, UI/upscale pipelines and world pipelines are built off that thread. CPU map preparation overlaps GPU discovery; world GPU finalization requires the device.
- Loading uses `ControlFlow::Wait`. A queued result followed by `EventLoopProxy::send_event(AppEvent::Loading)` wakes `user_event`; there are no polling simulation ticks. Results queued before the proxy/first resume are drained after window creation. The render thread can show the splash once ready; the window is initially blank.
- On success, `Engine::new(LoadedEngine, window)` consumes the completed load and activates the root world. The loading object is dropped, simulation timing is initialized, and `WaitUntil` starts regular ticks. Latest window dimensions are applied at activation. Initial actor construction and required sound decoding have already finished when the descriptor is published.
- Close during loading hides the window immediately and requests cancellation between stages. The event loop continues dispatching until already-started blocking work finishes and releases its resources; then it exits. A driver call cannot be forcibly aborted. Queued surface replies are dropped before joining a loader, preventing a shutdown wait on an unanswered main-thread request. Worker failures/panics become a terminal loading error, not an endless loading state. MainLoop retains the final window owner through cleanup.
- Headless waits for the same loader's result with no winit/window/GPU/rendering/sound device, constructs Engine, then runs the plain tick loop. Game logic is the same.

There is no game-mode argument. UISystem and NetSystem exist in both modes; the GPU UI renderer does not exist in headless mode.

**Module layout mirrors the architecture:** `src/engine/shared_systems.rs` + `src/engine/shared_systems/` (audio, global_settings, input, net, render, resources, time, ui) hold the shared systems; `src/engine/world.rs` + `src/engine/world/` (actors, messenger, physics, time, world_descriptor, static_object, effects) hold the per-world parts; `src/engine/loader.rs` (initialization), `src/engine/main_loop.rs` (application runner) and `src/engine/transform.rs` belong to the engine as a whole. `src/game_client_main.rs` is only the entry point (args, logger, allocator, auto-updater, `engine::load(start_level, backend, headless)` + `MainLoop::run(loading)`). See [repository map and placement rules](../AGENTS.md#repository-map).

**`EngineSharedSystems`** (`src/engine/shared_systems.rs`), once per application, all `pub` fields:
- `global_settings: GlobalSettings`: `user` (`user_settings.json`: sensitivity, volume, render scale, shadows, `target_render_frame_rate`; loaded next to the executable first, otherwise in the platform config dir; saved to the selected path) + `connection` (compiled in from `src/assets/settings/connection_settings.json`, overridable only by a `connection_settings.json` next to the executable)
- `time: GlobalTimeSystem`: engine tick timing, counters, wall clock (no session time)
- `input: InputSystem`: collected once per tick; actors read `shared.input.actions()`
- `net: NetSystem`: transport only (connection handles, WebSocket/WebRTC, raw packets)
- `resources`: the actor-facing `ResourceLoader` for the application cache
- `audio`, `ui`
- `gpu: Option<GpuContext>`, `render: Option<RenderScheduler>` (`None` in a ready headless engine; present in a ready graphical engine)
- `runtime`: tokio, 2 async workers (networking, resource I/O and settings persistence), plus its blocking pool for CPU/native work

**`World`** (`src/engine/world.rs`), one isolated simulation:
- `actors: Actors` (`world/actors.rs`)
- `messenger: Messenger` (`world/messenger.rs`): the message bus and the whole message pump
- `physics: PhysicsSystem`
- `time: WorldTimeSystem` (`world/time.rs`): the world's elapsed interval since its previous `simulate()`, subject to the simulation-time clamp
- `render: Option<WorldRenderSystem>`

`descriptor.give_me_world(config, &mut shared)` returns an active world. The descriptor owns parsed actor JSON, map geometry, shared pipelines and one ready set of independently constructed actors. The call synchronously constructs a replacement set from the parsed JSON, swaps it with the ready set under a short lock, then registers per-world systems and runs addition/start callbacks on the outgoing actors. The callbacks run after the lock is released. Rendering uses the caller’s current configuration. `World::simulate(&mut shared)` runs world time, actor ticks, message pump, physics, message pump and render data update.

During initial map preparation, constructors execute on a loading worker. They request their own assets and leave gameplay effects for activation. For example, `MachineGun` owns an optional typed sound-buffer handle and requests `embedded_asset!("sounds/machinegun_shot.wav")` in its constructor. The macro returns an `embedded/` key. Before compilation, `build.rs` scans Rust token trees under `src/` for direct `embedded_asset!("relative/path")` invocations and emits one `include_bytes!` per distinct key. Paths are relative to `src/assets/` and use forward slashes. The scan precedes macro expansion and includes literal markers in conditional and test code. The sound loader looks up the static bytes on a cache miss. Gameplay sound decodes can overlap during preparation. Headless actors skip these optional loads. Devices and actors spawned during gameplay can use the same request API. A sound played while its buffer is pending gets a silent source that can receive controls until the buffer is attached. Without a sound device (headless, or the device failed to open) or without a buffer, play calls return no source before constructing one.

**Typed resource loading.** Actors receive a concrete `ResourceLoader` with two calls. Constructors borrow it and tick callbacks use `shared.resources` directly. `load<T>(key, retention)` returns a Fyrox `Resource<T>` immediately. `load_media<T>` returns `None` in headless mode before lookup, I/O, pinning or dependency collection. In graphical mode it returns `Some(load(...))`, including a handle when loading fails. Each world preparation owns a local dependency vector. It lends that vector to a temporary loader during actor construction, then awaits the recorded handles, including cache hits. Activation uses the ordinary shared loader to refill the descriptor’s spare actors. A required GPU-only asset fails immediately in headless mode. A world supports CPU-only preparation.

```rust
// Shotgun constructor; the field's Option<SoundBufferResource> supplies the type.
shotgun_shot_sound: resources.load_media(embedded_asset!("sounds/shotgun_shot.wav"), Retention::Session),

// Trigger activation or transition. The tick polls the saved handle with try_read.
self.next_world = Some(shared.resources.load(&self.next_level_name, Retention::WhileUsed));

// EngineLoader starts the map before waiting for settings or the device.
let initial = resources.load::<WorldResource>(&initial_world, Retention::WhileUsed);
```

The engine owns the cache through a private `Arc<ResourceSystem>`. The ordinary loader shares this allocation for its lifetime. The engine updates the cache once per tick after `shared.pre_simulate()` and before world simulation. The manager shares pending work and ready data by logical path. Each asset type maps the key to the extension its Fyrox loader owns. A cache miss resolves catalog bytes or reads a file on Tokio, then runs the same decoder or parser for both sources. Blocking work uses the runtime's blocking pool. The Fyrox handle publishes readiness or failure directly to waiters. A tick uses `try_read` for one short inspection and can defer a pending or busy handle until the next tick. An unloaded handle has no active load and returns a terminal error on inspection, matching the result of awaiting it. Fyrox 1.0 maps every requested path to a UUID in its registry and saves the registry through the manager's `ResourceIo` after each change. `ResourceSystem` gives the manager `LoaderOwnedIo`, which neither reads nor writes, and marks the registry loaded without scanning a folder. The registry therefore stays in memory, a tick that requests a new path does no file I/O, and only the loaders read files. UUID/path entries survive cache expiry. For a long-running workload with observed registry growth, `Engine::prune_resource_registry()` can explicitly remove mappings for already evicted resources and return the number removed. It holds the manager state for the scan and runs only on an explicit call. See [registry retention](../issues/programming/resource-registry-retention/).

To add a file-backed texture, implement `Asset` for its data, set `REQUIRES_GPU` if the prepared value owns GPU data, and register its loader in `ResourceSystem::new`. The loader uses catalog-first lookup, decodes, awaits the published device and uploads. The actor can infer the type from its optional texture field and use `load_media`. Dynamic render targets have separate lifetime and update requirements.

A session request pins the shared handle until shutdown, including promotion after the initial request. Ordinary resources expire after 60 seconds of unused cache-update time. Startup and transitions load the same world key. The loading worker publishes device readiness before awaiting the initial map. Only the graphical finish of map preparation waits for the device. Closing the publisher before readiness fails the wait. Returning to a session-pinned map creates a fresh simulation from the retained descriptor. A final trigger with `async_preload` requests the destination on addition. Otherwise it requests the destination at transition time. In both cases it polls each tick and sends one replacement message on success or logs one error on failure. Older maps may retain the boolean `async_load` field with either value.

Engine tick (`Engine::tick`), target 60 Hz (`GlobalTimeSystem::new(60)`). Note: The average period is currently a bit longer than 16.(6) ms, see [engine-tick-rate](../issues/programming/engine-tick-rate/):

1. `shared.pre_simulate()` collects gamepad input, starts frame timing, updates input actions, polls the network and updates audio.
2. `resource_system.update(shared.time.prev_frame_duration())` advances cache expiry.
3. `root_world.simulate(&mut shared)`, then handle the root world's `OuterWorldMessage`s (quit, replace root world, ...)
4. render hotkeys (`i`/`O` root output scale, `P` shadows; persisted)
5. `shared.post_simulate()`: `net.flush`, UI buffers, `input.reset_input`, `time.end_of_frame`

Window-level hotkeys (`Esc`/`Enter`, `J`/`K` sensitivity, `N`/`M` volume, `Y`/`H` browser) are handled in `engine/main_loop.rs` and operate on `engine.shared`.

**Communication model:** actors communicate with other actors through Messenger; they can access shared systems directly through EngineSharedSystems. They push into their world's `Messenger`:
- broadcast `Message`s, direct `(ActorID, Message)` pairs
- `Command`s: `SpawnActor`, `SpawnControlledActor`, `RemoveActor`, `ChangeMyId` (an actor changes only its own id), `RemoveAllHolesAndEffects`
- Actor ids: random non-zero `u128`; `0` is reserved for the outer world. If an inserted actor or a `ChangeMyId` target takes an id that is in use, the previous holder moves to a fresh random id (the generator explicitly excludes the id reserved for the incoming actor); every moved actor broadcasts `IWasChangedMyId` from its previous id (the displaced actor's notification is queued first, so FIFO receivers never rename one reference twice), and `Actors::main_actor_id` keeps identifying the same actor. Queued messages are resolved by id at delivery time (a vacated id drops them, a reused id reaches its new holder). Details: [main actor ID collisions](../issues/programming/main-actor-id-collisions/).
- `send_to_outer_world(OuterWorldMessage)`: requests to the world's owner (the engine for the root world)

The capture-the-flag status identifies the carrier by its session peer ID. The flag resolves that peer in its own world on capture or when the player appears. Position requests then use the local actor ID and follow its rename notifications. A join snapshot can arrive before the remote doll spawns. [Carrier binding and message order](../issues/programming/flag-carrier-id-changes/)

`Messenger::process_messages` drains everything until empty in one FIFO order: broadcast messages, direct messages and commands are handled in the order they were sent, and items sent during handling are appended after everything already queued. Successive state updates therefore end with the last one sent (NetSession queues packets in arrival order); a direct message sent after `SpawnActor` reaches the new actor, one sent after `RemoveActor` is dropped, and `RemoveActor` followed by `SpawnActor` with the same id leaves the new actor. Storage is one `VecDeque` of items; `SpawnActor` boxes its `ActorWrapper` so a queue slot stays message-sized. The outer-world mailbox is FIFO too. Rationale: [message ordering](../issues/programming/message-ordering/). The owner-to-world API uses reserved sender id `0`. SessionController also uses it for some server-originated events; relayed player messages preserve the remote player ID, so network messages do not universally have sender `0`.

**Nested worlds:** an actor calls `descriptor.give_me_world(config, shared)`, calls `world.simulate(shared)` when it wants, talks to it with `send_broadcast_message`/`send_direct_message` and `take_outer_world_messages`, and controls rendering with `set_render_rate`, `set_output_resolution`, `set_output_scale`. `output_texture_id()` identifies the world's texture. Dropping the world releases its render-state ownership token; the render loop subsequently removes the expired slot.

### Actors (`src/actor.rs`, `src/actor/`)
- `trait Actor` callbacks get `(&mut self, world: &mut WorldContext, shared: &mut EngineSharedSystems)`:
  - `tick`, `recieve_message` (the misspelling is intentional and used everywhere), `on_added_to_world`, `on_world_started`
  - `recieve_message` gets `&Message`: Messenger keeps ownership of the popped message during delivery, so a broadcast is borrowed by every recipient instead of cloned per actor. A handler that stores or forwards payload data copies/clones only that data; it must not keep the reference beyond the callback.
  - `WorldContext` = `messenger`, `physics`, `time` (`world.time.delta()`), `main_actor_id`
  - also `get_physical_element`, `get_visual_element`, id handling
- `trait ControlledActor` covers player-like actors (camera, screen effects, team, spawn).
- `enum ActorWrapper` and its `impl Actor` are generated by the `actor_wrapper!` macro in `actor.rs`. **To add a new actor:**
  1. create `src/actor/<name>.rs` and add `pub mod` in `actor.rs`
  2. add one line `Name(Type),` to the `actor_wrapper!` invocation
  3. if the actor has messages, add a `SpecificActorMessage` variant
  4. if it is placed from maps, handle it in `parse_json_actors` (`src/engine/world/world_descriptor.rs`)
- `main_player/` is the main player: inner state, `constants.rs` (gameplay values, formerly `PlayerSettings`). `device/` holds weapons and tools (holegun, machinegun, shotgun, obstaclesgun, rotator); devices are not actors.
- Input: actors read `shared.input.actions()` directly (there is no `InputMaster`).

### Levels & assets
- Maps are JSON files in `src/assets/maps/`. The world resource loader resolves a name like `obstacle_course_levels/level-2` by trying these paths in order:
  1. `./<name>.json`
  2. `./assets/maps/<name>.json`
  3. `./src/assets/maps/<name>.json`
  4. `../../../src/assets/maps/<name>.json`

  Missing files fall through to the generated embedded catalog. A present but unreadable or malformed override reports its path. CLI names and trigger destinations follow the same order. The bare level name, its `.json` spelling and its bundled marker share a cache key. Distinct logical paths retain distinct identities. To ship another bundled map, add its literal marker to `src/engine/shared_systems/resources/source.rs`.
- WAV keys search the catalog first, then the corresponding disk path. A plain `sounds/name.wav` key and its bundled marker share a buffer.
- `old_maps_archive/` holds unused historical maps.
- Textures, skyboxes and shaders are embedded at compile time via `include_bytes!`/`include_str!`. Sounds go through fyrox-sound. The splash screen is `media/slice_4d_shooter_poster.png`.
- Settings: `GlobalSettings::load` asynchronously reads configuration and creates a default file if needed. `save()` queues a small snapshot for one async writer; writes remain ordered and do not block tick/window callbacks. Normal exit queues the final save and awaits `flush()` before runtime destruction. This waits for file writes, not power-loss durability (`fsync`). Gameplay values are constants, not settings.

### Rendering (`src/engine/shared_systems/render/`)

Backend policy: the wgpu instance explicitly permits Vulkan, DX12, Metal and browser WebGPU, and rejects explicit requests outside that set. Native CLI choices are `vulkan`, `dx12`, `metal`; WebGPU requires a browser build, which this project currently does not support. Cargo disables wgpu's default features and explicitly enables the retained backends plus `std`, `parking_lot` and `wgsl`; GLES/WebGL is excluded. The WebGL2 device-limit fallback was removed; default limits and the existing downlevel fallback remain. Adapter initialization logs the selected GPU and backend. See [OpenGL removal and rationale](../issues/programming/opengl-context-contention/).

Asynchronous CPU/GPU preparation and the existing separate simulation/render threads remain. No single GPU owner thread, GL-only serialization, or mandatory preload of every shader was introduced.

- `gpu_context.rs`: `GpuContext` (device, queue, shared layouts, sky box, shared buffers). World output textures use the fixed sRGB format `WORLD_OUTPUT_FORMAT`.
- `render_scheduler.rs`: `RenderScheduler` runs the render loop on a dedicated OS thread (`std::thread`), stopped by a `Shutdown` message + `join()` in `Drop`. Worlds register through a channel; each `WorldSlot` holds a `Weak<WorldRenderShared>` (atomics: render rate, packed resolution+scale) and the world's GPU objects. Per frame: skip if the GPU is busy (no frame queueing) or too early (the minimum frame duration comes from the `target_render_frame_rate` user setting, default 125, read once at creation and scheduled on a fixed timeline; frames are not interpolated, so frames between ticks redraw the latest tick data), encode due worlds in reverse registration order (children first when parents register before their children), upscale the presented world (or the splash) to the surface, UI, one submit, present. On an eligible frame without a drawable, the scheduler periodically submits queued simulation uploads with an empty command list. This uses the same idle-GPU check and a 100 ms minimum interval between upload-only submissions. A drawable on the next eligible frame can be presented immediately. The same upload path handles a pending surface replacement.
- `presentation.rs` owns the render thread’s surface, original adapter, configuration and pending size. The window thread retains the original instance and a sender for the render inbox. It forwards resizes during splash and play. On `Lost` the render thread drops the old surface and wakes winit once. The window thread creates the replacement for the current window and returns it and its physical size in the same FIFO as resizes. Before acquiring a frame, the render thread applies the latest nonzero size and checks replacement format compatibility with the existing pipelines. `Outdated` schedules reconfiguration. Close and shutdown can proceed while a request is outstanding.
- Actors keep using the same world, device resources and connections throughout recovery. After a frame is submitted and presented on the replacement, a later loss can request another replacement. A failed replacement, loss before its first presentation, or device loss reaches `MainLoop` during loading or play. The application exits with an error and flushes settings if already running. See [lost surface recovery](../issues/programming/lost-render-surface/).
- `world_render_system.rs`: `WorldRenderSystem` (per world, main thread) computes render data in `simulate()` and writes the buffers via `queue.write_buffer`. No application-level locks between the threads; multiple buffer writes are not an atomic snapshot (see [GPU data grouping](../issues/programming/gpu-data-grouping/)).
- `ui_renderer.rs` draws the HUD on top of the presented world (the scanner is re-bound to the presented world's buffers).
- WGSL shaders live in `shared_systems/render/shaders/`. Each level picks its main raymarch shader in `world_descriptor.rs`.
- `raymarch_shader_generator.rs` (~5k lines) bakes a static BSP tree of the map's geometry into the raymarch shader **at runtime** (`WorldPreparation::finish`, maps with `generated_shader_with_bsp_tree: true`). There are no pre-generated `*_with_bsp_tree.wgsl` files any more.
- `render_data/` splits static from dynamic GPU data.

### Physics (`src/engine/world/physics/`)
SDF-based 4D colliders, one `PhysicsSystem` per world:
- static, kinematic (players) and dynamic colliders
- areas (triggers)
- `common_physical_functions.rs` holds the SDF primitives

Math uses `glam::Vec4`. W is the 4th spatial axis.

### Networking
- `src/engine/shared_systems/net.rs`: `NetSystem` is a protocol-agnostic transport: `connect(ConnectionConfig) -> ConnectionHandle`, `state(&handle)`, `send(&handle, packet, reliability)`, `receive(&handle)`, `local_peer_id(&handle)`. WebSocket (matchmaking) and WebRTC via `matchbox_socket` (game server; reliable channel 0, unreliable channel 1).
- **Connection operations and errors:** operations through a handle return `Result<_, ConnectionError>`: `send(&handle, ..) -> Result<(), ConnectionError>`, `receive(&handle) -> Result<Drain<Packet>, ConnectionError>`. `ConnectionError::Connecting` means the connection is not established yet; `ConnectionError::Disconnected(reason)` means it is closed. `receive` returns packets that arrived before a disconnect first and reports `Disconnected` only once they are taken, like a socket read returning data before end-of-stream: a server's last answer before it closes the connection is not lost. `Ok` from `send` means the packet was handed to the transport, not delivered. Queries are not operations and return plain values: `state(&handle) -> ConnectionState` for the owner's state machine and `local_peer_id(&handle) -> Option<u128>` (WebRTC only, `None` until assigned). Use each handle with the `NetSystem` that issued it. A foreign handle yields `Disconnected(UnknownConnection)` for state and operations and `None` for the local peer id.
- **Connection ownership.** Moving a `ConnectionHandle` transfers ownership. Cloning it shares the same connection and receive queue. Each caller releases its share by dropping the handle. At the start of its next tick, `NetSystem` removes entries whose last handle has gone and drops their transports before polling. A handle contains one private `Arc<()>`. Its allocation address identifies the entry, whose `Weak` keeps that address reserved until removal. For example, if a controller passes a clone to a nested world, dropping the controller leaves the nested world's connection usable. Application shutdown drops the registry itself.
- Dropping a WebSocket transport cancels its single I/O task through the event receiver, including a pending handshake, blocked write or close. Each WebRTC transport owns its Matchbox socket task and aborts it on drop, including during the initial signaling handshake. Matchbox 0.15 native peer guards asynchronously close the peer connections and release their ICE sockets. WebRTC sends use the fallible enqueue operation. See [connection lifetime](../issues/programming/session-connection-lifetime/).
- `src/actor/session_controller.rs` + `session_controller/net_session.rs`: **all game-session logic** (press `G` to connect, matchmaking request, joining, decoding server messages into actor messages, connection UI, session/server time). Other actors send network data to it with `send_to_session_controller(messenger, from, SessionControllerMessage::...)` (a broadcast). The current game actors route session traffic through SessionController, so a world without one has no game session. This is a gameplay convention, not an engine restriction: actors can access shared transport directly.
- Messages are serialized with `alkahest`.
- Protocol types live in `game_server/src/client_server_protocol.rs`. Editing them affects both client and server and requires a compatible deploy.


## Current limitations and decision history

Simulation intervals over 0.35 s currently fall back to about one nominal tick; this is not an unrestricted real-time clock. See [world time semantics](../issues/programming/world-time-semantics/).

World removal unregisters rendering and releases its actors’ connection handles. Connections shared with surviving owners stay registered. The output texture resolver is deferred to [in-world screens](../issues/programming/in-world-4d-screens/). At the root, `WorldFinished` is ignored and the world keeps running; only `QuitApplication` exits the application. A nested world's owner chooses its own response to `WorldFinished`: [root WorldFinished decision](../issues/programming/root-world-finished/).

The [refactoring task](../issues/programming/engine-refactoring-world-isolation/) links its historical context: Part I for rationale, §19 for discussions, §24.9 for implementation rounds. Current proposals are recorded in the linked follow-up tasks; they are not implemented architecture.

Startup loading now lives in a separate EngineLoader and uses event notifications; see [implementation and verification](../issues/programming/nonblocking-engine-startup/). Actor activation remains synchronous, and supported native platforms still need runtime verification beyond the Linux environment used for this implementation.
