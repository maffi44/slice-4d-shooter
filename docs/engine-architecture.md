# Engine architecture reference

Implementation reference updated 2026-11-20. For working rules see [AGENTS.md](../AGENTS.md); for task status see the [open tasks](../issues/open-tasks.md).

## Engine architecture (`src/engine.rs`, `src/engine/main_loop.rs`)

The design, its rationale and history are in [engine-refactoring-world-isolation](../issues/programming/engine-refactoring-world-isolation/) (context Part I: rationale, Part III: historical target architecture; the main task tracks current status). Summary:

**`Engine`** = `root_world: Option<World>` + `shared: EngineSharedSystems` + the window. It contains no loading state or startup handles. The engine knows only the root world. Nested worlds are owned and simulated by actors.

Startup is separate from simulation ([decision and implementation context](../issues/programming/nonblocking-engine-startup/)):

```rust
let loading = engine::load(start_level, args.graphic_backend, args.headless);
MainLoop::run(loading);
```

- `engine::load` returns `EngineLoader`. It starts async map/settings loading and GPU instance discovery concurrently; a dedicated loading thread prepares shared systems (including audio/input/UI), coordinates dependencies and owns intermediate handles. The Tokio runtime ultimately moves into the ready shared systems.
- Graphical `MainLoop` owns the winit application. First `resumed` creates the window (duplicate resumes are guarded). The loader requests a surface through its result channel; only window/surface creation happens in the event-loop callback, preserving Metal's main-thread requirement. Adapter/device, shared GPU resources, UI/upscale pipelines and world pipelines are built off that thread. CPU map preparation overlaps GPU discovery; world GPU finalization requires the device.
- Loading uses `ControlFlow::Wait`. A queued result followed by `EventLoopProxy::send_event(())` wakes `user_event`; there are no polling simulation ticks. Results queued before the proxy/first resume are drained after window creation. The render thread can show the splash once ready; the window is initially blank.
- On success, `Engine::new(LoadedEngine, window)` consumes the completed load and activates the root world. The loading object is dropped, simulation timing is initialized, and `WaitUntil` starts regular ticks. Latest window dimensions are applied at activation. Initial actor construction and required sound decoding have already finished when the descriptor is published.
- Close during loading hides the window immediately and requests cancellation between stages. The event loop continues dispatching until already-started blocking work finishes and releases its resources; then it exits. A driver call cannot be forcibly aborted. Queued surface replies are dropped before joining a loader, preventing a shutdown wait on an unanswered main-thread request. Worker failures/panics become a terminal loading error, not an endless loading state. MainLoop retains the final window owner through cleanup.
- Headless waits for the same loader's result with no winit/window/GPU/rendering/sound device, constructs Engine, then runs the plain tick loop. Game logic is the same.

There is no game-mode argument. UISystem and NetSystem exist in both modes; the GPU UI renderer does not exist in headless mode.

**Module layout mirrors the architecture:** `src/engine/shared_systems.rs` + `src/engine/shared_systems/` (audio, global_settings, input, net, render, resources, time, ui) hold the shared systems; `src/engine/world.rs` + `src/engine/world/` (actors, messenger, physics, time, world_descriptor, static_object, effects) hold the per-world parts; `src/engine/loader.rs` (initialization), `src/engine/main_loop.rs` (application runner) and `src/engine/transform.rs` belong to the engine as a whole. `src/game_client_main.rs` is only the entry point (args, logger, allocator, auto-updater, `engine::load(start_level, backend, headless)` + `MainLoop::run(loading)`). See [repository map and placement rules](../AGENTS.md#repository-map).

**`EngineSharedSystems`** (`src/engine/shared_systems.rs`), once per application, all `pub` fields:
- `global_settings: GlobalSettings`: `user` (`user_settings.json`, loaded next to the executable first, otherwise in the platform config dir; saved to the selected path) + `connection` (compiled in from `src/assets/settings/connection_settings.json`, overridable only by a `connection_settings.json` next to the executable)
- `time: GlobalTimeSystem`: engine tick timing, counters, wall clock (no session time)
- `input: InputSystem`: collected once per tick; actors read `shared.input.actions()`
- `net: NetSystem`: transport only (connection handles, WebSocket/WebRTC, raw packets)
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

During initial map preparation, constructors execute on a loading worker. They request their own assets and leave gameplay effects for activation. For example, `MachineGun` owns an optional typed sound-buffer handle and requests `embedded_asset!("sounds/machinegun_shot.wav")` in its constructor. The macro supplies a logical key and embedded bytes from one literal. A scoped request context collects the handles requested during this preparation. Decodes can overlap and readiness includes their completion. Headless skips audio requests. Devices and actors spawned during gameplay can use the same request API. A sound played while its buffer is pending gets a silent source that can receive controls until the buffer is attached.

**Resource system design intent.** An actor requests an asset by key and immediately owns a typed handle. The resource system owns cache lookup, loading, sharing, lifetime policy and publication of success or failure. The consumer chooses when to use the result. A tick can call `handle.try_read(...)` to obtain a value from ready data under one short lock. Pending work or a busy slot can wait until the next tick. A completed load error is a separate outcome. A caller performing asynchronous preparation can await the same handle.

A session request pins its resource until shutdown. Ordinary resources expire after 60 seconds of unused cache-update time. Startup registers its descriptor under the same key as transitions so CPU map preparation can overlap device discovery. A final trigger with `async_preload` requests the destination on addition. Otherwise it requests the destination at transition time. In both cases it polls each tick and sends one replacement message on success or logs one error on failure. Older maps may retain the boolean `async_load` field with either value.

Engine tick (`Engine::tick`), target 60 Hz (`GlobalTimeSystem::new(60)`). Note: The average period is currently a bit longer than 16.(6) ms, see [engine-tick-rate](../issues/programming/engine-tick-rate/):
1. `shared.pre_simulate()`: gamepad input, `time.start_of_frame`, frame input actions, `net.tick` (poll sockets)
2. `root_world.simulate(&mut shared)`, then handle the root world's `OuterWorldMessage`s (quit, replace root world, ...)
3. render hotkeys (`i`/`O` root output scale, `P` shadows; persisted)
4. `shared.post_simulate()`: `net.flush`, UI buffers, `input.reset_input`, `time.end_of_frame`

Window-level hotkeys (`Esc`/`Enter`, `J`/`K` sensitivity, `N`/`M` volume, `Y`/`H` browser) are handled in `engine/main_loop.rs` and operate on `engine.shared`.

**Communication model:** actors communicate with other actors through Messenger; they can access shared systems directly through EngineSharedSystems. They push into their world's `Messenger`:
- broadcast `Message`s, direct `(ActorID, Message)` pairs
- `Command`s: `SpawnActor`, `SpawnControlledActor`, `RemoveActor`, `ChangeMyId` (an actor changes only its own id), `RemoveAllHolesAndEffects`
- `send_to_outer_world(OuterWorldMessage)`: requests to the world's owner (the engine for the root world)

`Messenger::process_messages` drains everything until empty (`Vec::pop()`, LIFO currently; order-independent state updates are not guaranteed — see [message ordering](../issues/programming/message-ordering/)). The owner-to-world API uses reserved sender id `0`. SessionController also uses it for some server-originated events; relayed player messages preserve the remote player ID, so network messages do not universally have sender `0`.

**Nested worlds:** an actor calls `descriptor.give_me_world(config, shared)`, calls `world.simulate(shared)` when it wants, talks to it with `send_broadcast_message`/`send_direct_message` and `take_outer_world_messages`, and controls rendering with `set_render_rate`, `set_output_resolution`, `set_output_scale`. `output_texture_id()` identifies the world's texture. Dropping the world releases its render-state ownership token; the render loop subsequently removes the expired slot.

### Actors (`src/actor.rs`, `src/actor/`)
- `trait Actor` callbacks get `(&mut self, world: &mut WorldContext, shared: &mut EngineSharedSystems)`:
  - `tick`, `recieve_message` (the misspelling is intentional and used everywhere), `on_added_to_world`, `on_world_started`
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
- Maps are JSON files in `src/assets/maps/`. `WorldPreparation::load` resolves a name like `obstacle_course_levels/level-2` by trying these paths in order:
  1. `./<name>.json`
  2. `./assets/maps/<name>.json`
  3. `./src/assets/maps/<name>.json`
  4. `../../../src/assets/maps/<name>.json`

  It then falls back to maps embedded with `include_str!` (the list is near the top of `world_descriptor.rs`). To ship a new map inside the binary, add it to that list.
- `old_maps_archive/` holds unused historical maps.
- Textures, skyboxes and shaders are embedded at compile time via `include_bytes!`/`include_str!`. Sounds go through fyrox-sound. The splash screen is `media/slice_4d_shooter_poster.png`.
- Settings: `GlobalSettings::load` asynchronously reads configuration and creates a default file if needed. `save()` queues a small snapshot for one async writer; writes remain ordered and do not block tick/window callbacks. Normal exit queues the final save and awaits `flush()` before runtime destruction. This waits for file writes, not power-loss durability (`fsync`). Gameplay values are constants, not settings.

### Rendering (`src/engine/shared_systems/render/`)

Backend policy: the wgpu instance explicitly permits Vulkan, DX12, Metal and browser WebGPU, and rejects explicit requests outside that set. Native CLI choices are `vulkan`, `dx12`, `metal`; WebGPU requires a browser build, which this project currently does not support. Cargo disables wgpu's default features and explicitly enables the retained backends plus `std`, `parking_lot` and `wgsl`; GLES/WebGL is excluded. The WebGL2 device-limit fallback was removed; default limits and the existing downlevel fallback remain. Adapter initialization logs the selected GPU and backend. See [OpenGL removal and rationale](../issues/programming/opengl-context-contention/).

Asynchronous CPU/GPU preparation and the existing separate simulation/render threads remain. No single GPU owner thread, GL-only serialization, or mandatory preload of every shader was introduced.

- `gpu_context.rs`: `GpuContext` (device, queue, shared layouts, sky box, shared buffers). World output textures use the fixed sRGB format `WORLD_OUTPUT_FORMAT`.
- `render_scheduler.rs`: `RenderScheduler` runs the render loop on a dedicated OS thread (`std::thread`), stopped by a `Shutdown` message + `join()` in `Drop`. Worlds register through a channel; each `WorldSlot` holds a `Weak<WorldRenderShared>` (atomics: render rate, packed resolution+scale) and the world's GPU objects. Per frame: skip if the GPU is busy (no frame queueing) or too early (the render rate is capped at the simulation tick rate: the minimum frame duration is the tick period passed at creation, scheduled on a fixed timeline, because render data changes once per tick and frames are not interpolated), encode due worlds in reverse registration order (children first when parents register before their children), upscale the presented world (or the splash) to the surface, UI, one submit, present.
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
- `src/engine/shared_systems/net.rs`: `NetSystem` is a protocol-agnostic transport: `connect(ConnectionConfig) -> ConnectionHandle`, `state`, `send(handle, packet, reliability)`, `receive`, `close`, `local_peer_id`. WebSocket (matchmaking) and WebRTC via `matchbox_socket` (game server; reliable channel 0, unreliable channel 1).
- `src/actor/session_controller.rs` + `session_controller/net_session.rs`: **all game-session logic** (press `G` to connect, matchmaking request, joining, decoding server messages into actor messages, connection UI, session/server time). Other actors send network data to it with `send_to_session_controller(messenger, from, SessionControllerMessage::...)` (a broadcast). The current game actors route session traffic through SessionController, so a world without one has no game session. This is a gameplay convention, not an engine restriction: actors can access shared transport directly.
- Messages are serialized with `alkahest`.
- Protocol types live in `game_server/src/client_server_protocol.rs`. Editing them affects both client and server and requires a compatible deploy.


## Current limitations and decision history

Simulation intervals over 0.35 s currently fall back to about one nominal tick; this is not an unrestricted real-time clock. See [world time semantics](../issues/programming/world-time-semantics/).

World removal unregisters rendering but does not yet close session connections: [connection lifetime](../issues/programming/session-connection-lifetime/). The output texture resolver is deferred to [in-world screens](../issues/programming/in-world-4d-screens/). `WorldFinished` currently requests application exit at the root; its enum comment claiming it is ignored is stale.

The [refactoring task](../issues/programming/engine-refactoring-world-isolation/) links its historical context: Part I for rationale, §19 for discussions, §24.9 for implementation rounds. Current proposals are recorded in the linked follow-up tasks; they are not implemented architecture.

Startup loading now lives in a separate EngineLoader and uses event notifications; see [implementation and verification](../issues/programming/nonblocking-engine-startup/). Actor activation remains synchronous, and supported native platforms still need runtime verification beyond the Linux environment used for this implementation.
