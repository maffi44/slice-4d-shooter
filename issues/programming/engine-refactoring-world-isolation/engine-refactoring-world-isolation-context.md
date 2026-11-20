# Context: Engine refactoring: isolated, nestable worlds

[Current task](engine-refactoring-world-isolation.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

## Completion (2026-11-16)

The author accepted the refactoring as complete on 2026-11-16; both files were renamed with the `done-2026-11-16-` prefix (removed on 2026-11-20, when status moved to the task lists; see the [issue README](../../README.md#tracking-convention-history)). Follow-ups from the 2026-11-16 review stay in their own tasks (see the main task).

## Preserved history (before the 2026-11-16 documentation migration)

The following original issue is preserved in full. Its old filename rules, status summaries and “final” architecture headings describe their original rounds and are superseded by the current task and AGENTS.md where they disagree. Section numbers remain available for older references.

# Engine refactoring: Engine Shared Systems + isolated, nestable Worlds

- **Created:** 2026-11-12 · **Last updated:** 2026-11-16 (session wrap-up after round 10; under the author's review)
- **Branch:** `dev`
- **Status:** **Implemented (2026-11-12), follow-up rounds 7–10 done (2026-11-13)** on `dev`, **not committed**. Now: the author reviews the code manually, then accepts (commits) or gives more feedback. When accepted, rename this file to `done-YYYY-MM-DD-engine-refactoring-world-isolation.md` (historical instruction; the current naming rule uses the acceptance/completion date and renames the context too) (issue naming rule in `AGENTS.md`). See §24.9 (implementation log) and §24.10 (what still needs manual verification by the author).
- **Current state in short (for using this issue as context):**
  - Done: `EngineSharedSystems` + isolated nestable `World`s; render loop on its own thread (`RenderScheduler`); `NetSystem` = transport, session logic in `SessionController`; `GlobalSettings` (user settings in the platform config dir, connection settings embedded / next to the exe); `PlayerSettings` → constants; module layout mirrors the architecture (`engine/`, `engine/shared_systems/`, `engine/world/`); `Engine::new(headless).run(level, backend)` with the winit main loop inside the engine; crutch flags removed; three working client bins; zero compiler warnings; found bugs fixed (dolls' team-win effect, unbreakable static shapes packing, WinSparkle call restored); licenses restored (AGPL-3.0-or-later code, CC BY-SA 4.0 assets; files carry only the copyright line).
  - Open, outside this issue: the global allocator (`issues/programming/global-allocator/`); the exact engine tick rate (`issues/programming/engine-tick-rate/`); most maps don't load (`issues/programming/maps-out-of-sync-with-parser/`); obstacle-course maps lack `it_is_2d_3d_example` (§24.10); everything in §24.10 needs playtesting by the author.
- **Participants:** project author (Timofei), Claude (analysis & proposals)
- **Related issues:**
  - `issues/programming/resource-system/`: Resource System, split out of this issue
  - `issues/programming/network-authoritative-server-and-transport/`: future network work, out of scope here
  - `issues/programming/in-world-4d-screens/`: displaying a world's texture on a screen object in 4D space (formerly "Q5"), out of scope here
  - `issues/programming/global-allocator/`: blink-alloc findings and the allocator question, split out in round 8–9

This file keeps the full context of the task: motivation, analysis of the current code, the discussion, decisions and open questions. It is not a verbatim transcript, but it aims to keep every essential detail, constraint and reason. Update it as decisions are made, especially §17 (decision log), §18 (open questions) and §19 (discussion).

[accepted] Author's rule for issues (round 4): keep the **wide context**, including intermediate ideas, arguments and rejected variants. Don't delete superseded ideas; mark them [rejected] with the reason. "We don't save bytes."

Structure of this file (done in round 6):
- **Part I — Concept, architecture principles and rationale.** What we build and why. Start here.
- **Part II — Design discussion and decision history** (§1–§20, §23). Analysis of the old code, every topic as it was discussed (including superseded ideas, marked [rejected]), the decision log (§17), open questions (§18), the chronological discussion (§19), code findings (§20) and the broader engine review (§23). Later decisions override earlier ones; the decision log is the index.
- **Part III — Final target architecture** (§21 Rust structures, revision 3; §22 key operations).
- **Part IV — Technical specification and implementation plan** (§24), including the implementation log (§24.9).

---

# Part I — Concept, architecture principles and rationale

This part explains **what** the refactoring builds and **why** it's built this way. It's the condensed result of the whole discussion (rounds 1–6, see Part II for the history). If a later part and this part ever disagree, the decision log (§17) and Part III/IV are authoritative, and this part should be fixed.

## I.1 Goal and expected result

**Concrete goal.** An in-game tutorial: a screen inside a level shows *another world*: a 2D slice of a 3D world, as an analogy for how the player sees a 3D slice of 4D space. This needs **nested worlds**: a world whose actors own other worlds, each simulated independently and rendered into its own texture.

**Broader goal.** The engine moves from an experimental demo to an architecture suitable for long-term, full-scale development: clear responsibilities, systems passed in a uniform way, no hidden global coupling, ready for the next big tasks (Resource System, authoritative server, in-world screens, UI rewrite).

**Expected result of this refactoring ([accepted] author):**
- The existing game works **exactly as now**: it starts, loads the Capture the Flag map, and actors, physics, controls, rendering, networking and all mechanics keep working.
- Internally the engine follows the new structure: `EngineSharedSystems`, `World`, `Actors`, `Messenger`, `WorldDescriptor`, `RenderScheduler`, `GpuContext`, `GlobalTimeSystem` / `WorldTimeSystem`, `NetSystem` as a transport, `SessionController` owning the game session, `GlobalSettings`.
- It's technically possible to create nested worlds, simulate them independently, render them into separate textures, control their render rate, resolution and scale, and get their textures for other GPU pipelines.
- **Not** in this refactoring: displaying those textures on in-world 4D screens, the Resource System, the authoritative server, the UI and Input rewrites (separate issues; the architecture is prepared for them).

## I.2 Guiding principles (the author's, applied throughout)

1. **The simplest, most logical architecture.** No abstraction unless it's really needed. If a simple task needs many extra mechanisms, reconsider the architecture instead of adding mechanisms.
2. **Clear responsibility per system.** Each system/actor owns one concern. Example: the transport doesn't know game rules; the session actor doesn't know sockets.
3. **Remove locks that aren't needed.** Every lock that can be removed without unnecessary complexity should be removed, especially on performance-critical paths (rendering).
4. **Minimal input-to-photon and input-to-network latency.** Slice is a fast competitive shooter. No frame queueing; outgoing network messages are sent in the same tick they're produced.
5. **Keep established names and concepts** unless there's a strong reason to change them (`Messenger`, `Actors`, `simulate()`, `GlobalSettings`, `recieve_message`, …).
6. **Discuss and agree on the architecture first**, then make large changes.
7. **Don't mix the current task with other large changes** that can be done later; record them as separate issues instead.
8. **Gradual evolution.** Each implementation stage keeps the game playable.

## I.3 The core idea: Engine Shared Systems + isolated Worlds

The engine is split into two kinds of things:

- **`EngineSharedSystems`**: things that exist once per application and are shared by every world: global settings, global time, input, audio, UI, network transport, GPU context, render scheduler, async runtime.
- **`World`**: one isolated simulation: its actors (`Actors`), its message bus (`Messenger`), its physics, its own time (`WorldTimeSystem`), its render data and output texture (`WorldRenderSystem`).

`Engine` = `EngineSharedSystems` + one **root world** (+ the window). Nested worlds are owned by actors.

Every actor callback receives **two things**: `&mut WorldContext` (its own world's messenger, physics, time) and `&mut EngineSharedSystems`. This replaces today's 7–11 separate system parameters.

### Why split shared and per-world systems
- Some things are inherently single per application: one window, one GPU device and queue, one audio output, one keyboard/mouse, one network stack, one settings file. Duplicating them per world would be wrong (one GPU device is mandatory so that a parent can sample a child's texture).
- Other things must be isolated per world, or worlds would leak into each other: actors and their ids, messages, physics colliders, simulation time, render data. A nested demo world must not see the main level's actors, and vice versa.
- With this split, a nested world is "just another `World`" that receives the same `&mut EngineSharedSystems`. No special cases.

### Why every actor gets `&mut EngineSharedSystems`
- Any actor may own a nested world and must pass the shared systems on to its `simulate()`.
- It makes access uniform (Godot-style): input, audio, UI, net, settings are reachable from any actor in any world without threading parameters through.

## I.4 Rationale for the main decisions

### Nested worlds are owned and simulated by actors; the engine doesn't know the hierarchy
- **Ownership:** a nested world is an ordinary field of an actor. When the actor is dropped, the world is dropped. No registry, no ids for worlds, no lifecycle bookkeeping in the engine.
- **Simulation:** the owner calls `nested.simulate(shared)` inside its own `tick()`: every tick, every N ticks, or not at all. It knows best when its world needs to run (a screen far away or turned off can pause its world). A central simulation scheduler would need configuration and messages for things the owner can decide in one line.
- **The engine knows only the root world.** It doesn't need to know how many worlds exist or how they nest. Everything that must be global (rendering order, GPU) is solved locally by the render scheduler's registration (see below), not by a world tree in the engine.
- Consequence: a nested world's simulation rate is at most its owner's tick rate.

### Simulation ≠ rendering
- `simulate()` updates the world and writes its render data into GPU buffers. Drawing happens on the render thread at the rate the render scheduler assigns.
- They run at different rates already today (60 Hz simulation vs ~125 FPS render) and will become independently configurable later. The current coupling is accidental.

### Per-world time
- If a world is simulated every Nth tick but advanced by the engine's frame delta, it runs N times slower than real time. That's a systematic error, not jitter, so it doesn't average out.
- So each world has a `WorldTimeSystem` that computes **its own delta** = time since its previous `simulate()` (from the engine's frame timestamps, with today's clamp). For the root world this equals today's delta exactly.
- `GlobalTimeSystem` keeps only truly global time: engine tick timing, counters, application start, date/time.
- **Session time (server time) belongs to `SessionController`**, not to a global system: it's a property of one game session, and an application could in theory have several sessions.

### One shared render scheduler; render data belongs to worlds
- Independent renderers per world were considered first (round 1) and rejected because:
  1. there must be **one GPU device** (a parent samples a child's texture; resources can't cross devices);
  2. one device has **one in-order queue**: a heavy nested-world render queued before the main frame would delay the player's frame, which is exactly the latency the project avoids;
  3. a child's texture must be rendered **before** the parent pass that samples it; independent loops give no ordering.
- So there is **one render loop** (on its own OS thread) that, per frame, encodes all due worlds into **one encoder and one submit**, children before parents, then the presented world → window → UI. The "no frame queueing" check (`QueueEmpty`) stays valid.
- Each world owns its render data, buffers, pipelines and output texture; the scheduler only decides *when* to encode them. Worlds register through a channel and unregister automatically: the world holds an `Arc`, the scheduler a `Weak`; when the world is dropped, its slot disappears. No unregister call, no `Drop` impl.
- The **render loop runs on a dedicated OS thread**, not a tokio task: it's an infinite timing loop, async gives it nothing, and on tokio it permanently occupied one of the two workers. It's stopped explicitly on shutdown (a `Shutdown` message + `join()`), because dropping a `std::thread::JoinHandle` doesn't stop a thread.
- **`GpuContext` is a separate shared field.** It's infrastructure for creating GPU resources (device, queue, output format, layouts), while the scheduler is the system that executes rendering. Two responsibilities, two fields (author's decision).

### Avoiding locks and extra layers
- Render rate and output resolution/scale are **atomics** (`AtomicU32`, packed `AtomicU64`) read by the render thread each frame. One packed store keeps resolution and scale consistent. `Relaxed` ordering is enough for standalone values.
- GPU objects of a world are **moved into the render thread's slot** at registration; the main thread keeps only buffer handles for `queue.write_buffer` (thread-safe). So there is no `Mutex<WorldGpu>` at all (wgpu 27 handles are cheap clones).
- Registration goes through an `mpsc` channel; the render loop owns its slots exclusively.
- `InputSystem` loses its `Arc<Mutex<…>>` (input is single-threaded).
- No traits or generic frameworks where a struct with `pub` fields does the job (`EngineSharedSystems` is plain fields; messages are hand-written enums).

### Communication through `Messenger`, with hand-written enums
- Inside a world, actors never mutate each other or systems directly; they send broadcast/direct messages and commands through the world's `Messenger`. This already works well and is kept.
- **All message delivery, distribution and command execution live in `Messenger`.** `World` only calls it. Today this logic is spread over `engine.rs` and `World`.
- **Between worlds** it's deliberately simple: a world sends requests outward through its `outer_world_mailbox` (`OuterWorldMessage`: quit, replace the root world, "I'm finished", game-specific); the owner pushes messages inward into the nested world's ordinary buffers with sender id 0. The receiver of a request (engine or owner actor) decides what to do.
- Actors may change **only their own id** (`ChangeMyId`), never someone else's.
- No generic message system, no reflection, no type-erased payloads. Adding a message = adding an enum variant.
- Message order inside a tick stays as today (`Vec::pop()`, LIFO). Actor logic is written as if everything in one tick happens at once, so order shouldn't matter; `pop()` is fast and simple.

### `WorldDescriptor` vs `World`
- Creating a level today compiles shaders and creates pipelines: slow. Preloading and async creation are needed.
- A **`WorldDescriptor`** is the immutable, shareable (`Arc`) result of the slow work: parsed map, generated shaders, created pipelines, actor descriptions. It can be loaded in advance, asynchronously, and cached (later by the Resource System).
- **`World::new(&descriptor, config, shared)`** is cheap and contains *all* per-instance creation: own systems, actors (built by the descriptor), registration in the render scheduler, initial messages.
- Why not cache a pre-built `World` instead: a world isn't shareable (its own actors, physics state, GPU buffers), and its construction has side effects on shared systems (render registration, sounds, UI, network). Built during preloading, those would fire while the player is still in the menu.
- The descriptor **creates the actors** (`create_actors()`); `World` doesn't know about JSON. A `World` **doesn't keep** its descriptor: needed values are copied into its systems; whoever wants to recreate the world keeps the descriptor.
- Replacing the root world: a world prepares the next descriptor in advance and sends `OuterWorldMessage::ReplaceRootWorld(Arc<WorldDescriptor>)`. No engine-level "load level" commands.

### Game-session networking in `SessionController`; `NetSystem` is a transport
- WebRTC/Matchbox was chosen for a browser version that's no longer a priority. The game drifted into client-server with peer-to-peer-style messages relayed by the server.
- `NetSystem` becomes a **transport**: create connections, report their state, send/receive packets, several independent connections via **connection handles**. It knows nothing about the game protocol.
- **All game-session logic moves into the `SessionController` actor** (connection flow, matchmaking request, decoding server messages into actor messages, connection UI, session time). Other actors send their network data to `SessionController` as ordinary messages (variant V1).
- Why V1: incoming decoding moves there in any variant (the big part); latency is identical (same tick); nested worlds are naturally isolated (no session controller → no network); and protocol knowledge is concentrated in one actor, which is exactly what the future authoritative server needs.

### Input: one shared system, read directly (Godot-style)
- `InputSystem` collects input once per tick; any actor of any world reads `shared.input` (mainly input actions).
- `InputMaster` (`LocalMaster`/`RemoteMaster`) and `set_input_to_controlled_actors` are removed: `RemoteMaster` was never implemented, `LocalMaster` only cached a copy of the frame's actions. The question "should nested worlds get input" disappears: an actor that needs input reads it.

### Settings: constants + `GlobalSettings`
- `PlayerSettings` mixed `MainPlayer` gameplay values with app settings, and a local `settings.json` could override gravity, speeds and damage (a cheat vector). Gameplay values become **Rust constants**; real app settings go into **`GlobalSettings`** in the shared systems.
- **User preferences** (sensitivity, volume, resolution scale, shadows) are persisted in the user's platform config directory and saved when changed. **Connection data** (matchmaking URL, TURN) is compiled in and overridable only by a file next to the executable, so new releases can change servers without stale values in users' files.

### Splash / loading screen without a `World`
- While the initial world loads, the render thread draws a small splash component (logo; an animated indicator can be added later because it gets the time each frame). It keeps animating while the main thread is busy.
- The initial world loads **asynchronously** (`pending_root_world`), because blocking winit's event handling for seconds makes Windows/macOS mark the window as "Not Responding".
- The initial world is an `Engine::new` parameter: the CTF map now, the main menu later.

### Headless
- Headless = no window → `gpu: None`, `render: None`; worlds have no render system; texture ids never resolve. Game logic is identical. `HeadlessEngine` disappears.

## I.5 What is deliberately deferred, and why

| Deferred | Where | Why not now |
|---|---|---|
| Showing a world's texture on in-world 4D screens | `issues/programming/in-world-4d-screens/` | Needs its own design (depth from SDF pass, 4D quads, slicing under 4D rotation). The refactoring only has to provide the texture. |
| Resource System (typed path requests, cache, async handles) | `issues/programming/resource-system/` | Separate design (`fyrox-resource` spike). For now `WorldDescriptor::load` on `spawn_blocking` + `JoinHandle`, the pattern already used. |
| Authoritative server, dropping emulated P2P, UDP/TCP | `issues/programming/network-authoritative-server-and-transport/` | Protocol + server rewrite. The refactoring makes it easier (transport abstraction, protocol knowledge in `SessionController`). |
| UI rewrite (Fyrox UI, actor-owned UI elements) | later | Large; unrelated to world isolation. |
| Input rewrite (configurable actions) | later | Actions stay hard-coded; only `InputMaster` is removed. |
| Removing `SpawnControlledActor` | later pass | The author remembers having reasons for it; not worth the risk now. |
| FIFO message order | not planned | Logic shouldn't depend on order; `pop()` is fast. |
| `Option<ActorID>` + `expect` everywhere, panics as error handling, misspellings | later | Large noisy changes unrelated to the goal. |

---

# Part II — Design discussion and decision history

The sections below were written during the discussion (rounds 1–6) and are kept as the design history. Section numbers are stable references. Where a section's early content was superseded, it's marked [rejected] and the later decision is stated next to it.

## 1. Motivation

Slice 4D Shooter is a fast competitive multiplayer arena shooter in 4D space. The author has developed it by hand for about 2.5–3 years with practically no AI-generated code, and is now starting to use AI assistance to speed up development.

**The concrete goal of this refactoring is an in-game tutorial.** Inside a level there will be an in-game *screen* that shows **another game level**. It explains 4D space through the analogy 2D→3D ≈ 3D→4D:
- In the game, the player perceives 4D space through a **3D slice**.
- The screen shows the same situation one dimension lower: a **2D slice of 3D space**.
- The player walks up to the screen, presses a key, and sees a demonstration of the action they must perform, in the simpler 2D→3D analogy.

This requires **nested Worlds**: a `World` whose actors can own other `World`s, each rendered into its own texture.

The repo already has 2D/3D content: `map_2d_3d.json`, `raymarch_shader_for_2d_3d_example*.wgsl`, `player_for_2d_3d_example.rs`, and `Level::it_is_2d_3d_example`.

Besides nested worlds, the refactoring also cleans up the engine's structure:
- how systems are passed around
- `NetSystem` as a transport layer (§13)
- settings (§14)

---

## 2. Current architecture (analysis of the live code, 2026-11-12)

### 2.1 Core structures

| Component | File | Notes |
|---|---|---|
| `Engine` | `src/engine.rs` | Owns `render, input, physic, time, world, engine_handle, net, audio, ui, effects, settings`, a tokio `Runtime` (2 workers) and `level_async_load_handler`. |
| `HeadlessEngine` | `src/engine.rs` | A near-copy of `Engine` without render. |
| `World` (really an actor container) | `src/engine/world.rs` | `level`, `actors: HashMap<ActorID, ActorWrapper>`, `main_actor_id`, `players_settings`, `preloaded_levels` |
| `EngineHandle` | `src/engine/engine_handle.rs` | Buffers: broadcast messages, direct messages, commands. |
| Message pump | `engine.rs` | `send_messages_and_process_commands` loops until all buffers are empty. `execute_command` handles `SpawnActor`, `RemoveActor`, `NetCommand(..)`, `LoadNewLevelSync/Async`, `PreloadNewLevelAsync`, `RemoveAllHolesAndEffects`, … |
| `Actor` trait | `src/actor.rs:75` | `tick(&PhysicsSystem, &mut EngineHandle, audio, ui, time, effects, delta)`. `net` and `input` are not passed to actors. |
| `ActorWrapper` | `src/actor.rs:169` | ~30 variants; each `Actor` method is a big `match`. |
| `PhysicsSystem` | `src/engine/physics.rs` | `set_new_level(&World)`, `process_physics(&mut World, ..)`, raycasts. Already logically per-world. |
| `EffectsSystem` | `src/engine/effects.rs` | An empty struct with one helper, `spawn_wave`, which spawns an actor via `EngineHandle`. Not a real system. |
| `TimeSystem` | `src/engine/time.rs` | Frame timing (`get_prev_frame_duration`) and server time. Target 60 Hz. |
| `NetSystem` | `src/engine/net.rs` (~2000 lines) | Transport **and** game-session logic, see §13.1. |
| `InputSystem` | `src/engine/input.rs` | `set_input_to_controlled_actors(&mut World, ..)` writes input into every `ControlledActor` with `LocalMaster`. |
| `Level` | `src/engine/world/level.rs` | Parsed map JSON **plus pre-built GPU pipelines** (`main_shader_render_pipeline`, `minimap_shader_render_pipeline`), built via `MainShaderRenderPipelineBuilderKit` in `Level::load_level` (can run on tokio). |
| `PlayerSettings` | `src/actor/main_player/player_settings.rs` | Mixes `MainPlayer` gameplay constants with global app settings, see §14. |
| `Settings` | `src/engine/settings.rs` | Wraps `PlayerSettings`; only mutates mouse sensitivity (`Arc<Mutex<f32>>`). |

### 2.2 Current frame order (`src/main_loop.rs`)

1. `input.collect_gamepad_button_input()`, then `time.start_of_frame()`
2. if the async level load finished, `set_new_level(..)`
3. `net.tick(..)`
4. `input.set_input_to_controlled_actors(world, net)`
5. `world.tick(..)` (actors)
6. `send_messages_and_process_commands(..)`
7. `physic.process_physics(..)`
8. `send_messages_and_process_commands(..)`
9. `render.process_player_input(..)`, then `render.send_data_to_renderer(world, time, ui)`
10. `input.reset_input()`, then `time.end_of_frame()`

### 2.3 Current renderer (`render.rs`, `render/renderer.rs`, wgpu 27.0.1)

- `RenderSystem` (main thread) computes `RenderData` from the `World` and writes GPU buffers via `queue.write_buffer(..)`.
- `Renderer` lives in `Arc<Mutex<Renderer>>` in **a separate tokio task**. Each iteration does `try_lock()` and `render()`; on surface errors it calls `resize`; if the iteration took less than 3 ms, it sleeps ~3.1 ms.
- `Renderer::render()`:
  1. `device.poll(PollType::Poll)`; if the status is not `QueueEmpty`, skip. This is the **no frame queueing** check.
  2. Pacing: `target_frame_duration = 0.008` s, about **125 FPS**, decoupled from the 60 Hz simulation.
  3. One encoder:
     - raymarch pass into `raymarch_target_texture` (scaled by `raymarch_target_texture_scale_factor`, keys `i`/`O`)
     - optional xwz-minimap pass
     - upscale pass to the surface
     - UI pass
  4. One `submit`, `pre_present_notify`, `present`. `PresentMode::AutoNoVsync`.
- The pattern "render the slice into an offscreen texture, then use it" already exists: `raymarch_target_texture` is effectively the world's output texture.

### 2.4 Why there is no frame queueing [accepted]

Slice is a fast competitive shooter. The author experimented with queueing frames: output gets smoother, but **input-to-photon latency** grows, which is unacceptable. The renderer always renders the latest state and never submits while the GPU is busy.

### 2.5 Existing WIP refactor (untracked files)

Untracked, unwired, uncompiled modified copies of live modules:
- `src/engine/shared_systems.rs` + `shared_systems/`
- `src/engine/world/{actors,effects,geometry,level_descriptor,messenger,physics,render}.rs` + subdirs
- `src/engine/main_loop.rs`, `src/engine/transform.rs`

Names introduced there that are worth keeping:

| Old | WIP name |
|---|---|
| `EngineHandle` | `Messenger` |
| `World` (actor container) | `Actors` |
| `PlayerSettings` | `GameConfig` |
| `Settings` | `GlobalSettings` |
| `level.rs` | `level_descriptor.rs` |
| `static_object.rs` | `geometry.rs` |
| — | `EngineSystems` in `shared_systems.rs` |

[accepted] Decided:
- Claude may reuse or drop the WIP files and **may delete them without further approval**.
- The established names must be kept.

[proposed] Plan:
- Implement from the live (compiling) code, using the WIP files as a naming and structure reference.
- Delete them during stage 1, once nothing useful is left in them. Don't delete them before implementation starts; they are still a useful reference.
- Settings naming changes in §14: `GameConfig` disappears.

---

## 3. Target architecture: overview

```
Engine
├── root_world: Option<World>        // None while the initial world loads → splash (§21.10)
├── shared: EngineSharedSystems
│     global_settings: GlobalSettings,   // §14
│     time: GlobalTimeSystem,            // §5.3
│     input, net, audio, ui,
│     render: Option<RenderScheduler>,   // None = headless engine; owns the render OS thread
│     runtime (tokio)
│     (resources: ResourceSystem — see the Resource System issue, later)
└── window

World
├── actors: Actors                   (former `World` actor container)
├── messenger: Messenger             (former `EngineHandle` + outer-world mailbox)
├── physics: PhysicsSystem
├── time: WorldTimeSystem            // per-world delta (§5.3)
├── render: Option<WorldRenderSystem>   // per-world render data + handle to its render slot
└── descriptor: Arc<WorldDescriptor>    // immutable prepared data (§12)
   fn new(descriptor, config, shared) -> World
   fn simulate(&mut self, shared: &mut EngineSharedSystems)

(The full Rust draft is in §21.)
```

---

## 4. Shared vs World systems

| System | Category | Status / notes |
|---|---|---|
| Audio, UI, Input, Net | Shared | Accepted |
| `GlobalSettings` | Shared | [accepted] (§14) |
| tokio `Runtime` | Shared | Proposed |
| Time | **Both** | done in round 4: `GlobalTimeSystem` (shared: engine tick, start time, clock, server time) + `WorldTimeSystem` (per world: its own delta). See §5.3. |
| **Render Scheduler** (GPU context, surface, upscale, UI renderer, render thread) | Shared | Accepted |
| Resource System | Shared | [accepted] (separate issue) |
| Physics, Messenger, Actors | World | Accepted |
| World render system (pipelines, buffers, bind groups, output texture, `RenderData`) | World | [accepted] updated in `simulate()`, submitted by the scheduler |
| Effects | dissolved into a helper | [proposed] (`EffectsSystem` is an empty struct; `spawn_wave` becomes a free function taking `&mut Messenger`) |

---

## 5. `World::simulate()`, contexts, delta

### 5.1 Two-stage main loop (+ post stage for net) [accepted]

The current main loop splits into:
1. a **tick of the Engine Shared Systems** (collect input, net receive, time, …)
2. **`root_world.simulate(&mut shared)`**: one full world tick (actors, messages and commands, physics, messages again, render-data update; order taken from §2.2 and adapted)
3. [accepted] **a post-simulate step for shared systems**, at least `NetSystem`, so outgoing messages produced during simulation go out **immediately**, without waiting for the next engine tick (latency, see §13.5)

The name is `simulate()`, not `tick()` [accepted].

[proposed] Sketch:

```
Engine frame:
  shared.pre_simulate()      // input collect, time.start_of_frame, net receive/poll
  root_world.simulate(&mut shared)
  shared.post_simulate()     // net flush, input.reset, time.end_of_frame
  wait until next tick
```

### 5.2 Actor signature [accepted]

- Every actor receives **one** `&mut EngineSharedSystems` [accepted]. This is needed at least because any actor may own a nested `World` and must pass the shared systems on.
- Per-world systems (Messenger, Physics, …) are passed alongside, possibly bundled into a context struct.

[proposed] Shape:

```rust
fn tick(&mut self, world: &mut WorldContext, shared: &mut EngineSharedSystems)
// WorldContext: { messenger: &mut Messenger, physics: &PhysicsSystem, delta: f32, render info for creating child worlds, ... }
```

Changing signatures is explicitly fine [accepted]. This is a large mechanical change: ~30 actors plus every `ActorWrapper` match.

### 5.3 Time: `GlobalTimeSystem` + `WorldTimeSystem`, delta for nested worlds (done in round 4)

[accepted] **Decision (round 4).** The author agreed with the nested-delta problem described below, and split time into two systems:
- **`GlobalTimeSystem`** (in `EngineSharedSystems`): time information not tied to a world, e.g.
  - total number of engine ticks
  - application / session start time
  - current date and time
  - other general time information (may not be very needed yet, but useful long-term)
- **`WorldTimeSystem`** (one per `World`): the world's own simulation time, mainly **its own `delta`** = time elapsed since the previous `simulate()` of this world.

Author's assumption: the existing `TimeSystem` mostly becomes `WorldTimeSystem`, maybe minus main-loop-related functions; the exact split to be decided after studying the code.

[proposed] Claude's correction after studying `src/engine/time.rs` (round 4): it's mostly the other way round. Almost everything in today's `TimeSystem` is **engine-loop** timing, so it goes to `GlobalTimeSystem`. `WorldTimeSystem` is small and new.

| Current `TimeSystem` item | Used by | Goes to |
|---|---|---|
| `target_frame_duration` (60 Hz) | `main_loop.rs` (`ControlFlow::WaitUntil`) | `GlobalTimeSystem` |
| `start_of_frame` / `end_of_frame`, `timestamp_of_start_of_current_frame` | main loop | `GlobalTimeSystem` (`frame_start()`) |
| `prev_frame_duration` + `get_prev_frame_duration()` (clamped: > 0.35 s → 1/60) | main loop (physics delta, actor delta, hotkeys J/K/N/M) | `GlobalTimeSystem` keeps the engine value (hotkeys use it); world delta is computed by `WorldTimeSystem` with the **same clamp** |
| `frame_counter`, `timestamp_of_main_loop_start`, `average_frame_duration`, `current_frame_duration` | stats | `GlobalTimeSystem` |
| `set_server_time` / `get_server_time` | `NetSystem` sets it; `MainPlayer` / `PlayerFor2d3dExample` read it (9 uses) **only to timestamp outgoing network messages** | done in round 5: **`SessionController`**, not `GlobalTimeSystem` (see below) |
| — (new) | | `WorldTimeSystem { last_simulate_frame_start, delta, simulate_counter, world_time }` |

[proposed] `WorldTimeSystem::begin_simulate(&GlobalTimeSystem)` is called first in `World::simulate`:
- delta = `global.frame_start() − self.last_simulate_frame_start` (first simulate: the engine's previous frame duration)
- same clamp as today (> 0.35 s → 1/60 s), so resuming after a long pause doesn't produce a huge step
- for the root world this equals exactly today's `get_prev_frame_duration()`, so behaviour is unchanged
- using the engine's frame-start timestamp (not `Instant::now()`) keeps all worlds of one engine tick on the same clock

~~Server time note: it's conceptually per-connection (per game session). There is one game session per application, so keeping it in `GlobalTimeSystem` is the simplest option.~~ [rejected] superseded in round 5.

[accepted] **Round 5 (author): session time belongs to `SessionController`.** Everything related to a concrete game session (server time, timestamps, offsets and other synchronization parameters) is encapsulated in the `SessionController` actor, which stores and updates it. Reason: an application could in theory have several game sessions, and session time in a global system would be the wrong split of responsibility. `GlobalTimeSystem` provides only truly global time information. Server time in today's shared `TimeSystem` is a legacy of the raw architecture.

[proposed] This works out especially well in the code: all 9 reads of `get_server_time()` (in `main_player.rs` and `player_for_2d_3d_example.rs`) only **stamp outgoing messages** (`TryToReturnMyFlag(time)`, `TryToGetOpponentsFlag(time)`, `TryToGetScore(time)`, `TryToGetMoveWBonus(time)`, the player-state message). With V1 all of them pass through `SessionController`, so **`SessionController` adds the timestamp when it builds the network message**, and the actors no longer need server time at all. The protocol (`client_server_protocol.rs`) stays unchanged; the timestamp is taken a few microseconds later, in the same tick.

#### History of this topic

The author's view (round 3):
- Delta lives in `TimeSystem` (`shared.time`).
- Like in other engines, it is the duration of the **previous** frame, because the current frame's duration isn't known yet; deviations average out over time.
- Nested worlds can simply read the same delta from the shared systems.

Claude's note: the author's reasoning about "previous frame duration" is correct, but it addresses a different issue. The problem with nested worlds is **not** predicting the current frame:
- If an owner calls `nested.simulate()` only every 2nd engine tick, each call still sees `shared.time.delta` ≈ one engine frame (~16.7 ms), while ~33 ms of real time passed between this world's consecutive simulations.
- The world then advances half as much simulated time as real time passed, i.e. it runs at half speed. That is a systematic error, not jitter, so it doesn't average out.
- If an owner pauses simulation for 5 s and resumes, the world just continues (which may be desired).

[proposed] Minimal fix that keeps the `simulate(shared)` signature (no extra parameter):
- `World` remembers the timestamp of its last `simulate()` call.
- It computes its **own delta** = `shared.time.frame_start − self.last_simulate_frame_start`, clamped to a max (e.g. 100 ms), so a resume after a pause doesn't produce a huge step.
- Actors read `world.delta` from the `WorldContext`, not `shared.time.delta`.
- For the root world this equals the engine delta, so behaviour is unchanged. For a world simulated every N ticks it's N frames' worth, so it runs at real-time speed.

(Optional later: a per-world `time_scale` for slow-motion demos.)

~~[open] Accept this, or deliberately keep "nested worlds advance by the engine delta per simulate call"?~~ → resolved in round 4: per-world delta in `WorldTimeSystem` [accepted].

### 5.4 Messenger [accepted]

`Messenger` = former `EngineHandle` + outer-world mailbox (§7).

---

## 6. Nested worlds [accepted]

- The engine creates the root `World` at startup.
- **Nested `World`s are owned by actors** (a field of an actor). The engine does **not** track worlds or their count.
- **The owner controls simulation**: it calls `nested.simulate(shared)` inside its own `tick()` every tick, every N ticks, or not at all. **No simulation scheduler.**
- A nested world's simulation rate is at most its owner's tick rate, and can be lower.
- **Simulation ≠ rendering.** `simulate()` updates render data and GPU buffers; drawing happens at the rate the Render Scheduler assigns (§9).

---

## 7. Communication between worlds [accepted]

- Encapsulated in `Messenger`, kept as simple as possible. Hand-written enums, as for actor messages; no generic message system for now.
- **Outward (world → owner):**
  - `Messenger.outer_world_mailbox: Vec<OuterWorldMessage>` (hand-written enum)
  - actors call something like `messenger.send_to_outer_world(msg)`
  - after `nested.simulate(..)` the owner drains it
  - for the root world, the **engine** drains it
- **Inward (owner → world):** the owner pushes messages into the nested world's existing broadcast/direct buffers before `simulate`. **Sender id = 0** [accepted] (actor ids of different worlds are independent namespaces).
- **Root world → engine requests** [accepted]. Messages are *requests*; the receiver (engine or owner actor) decides what to do. Examples:
  - quit the application
  - **replace the root `World`** with another one (§12.4)

---

## 8. Input [accepted]

- Input System is shared and collected once in the shared tick. Its state is available to actors of **any** world via `EngineSharedSystems` (like Godot). No special restrictions for nested worlds.
- The current `InputMaster` / `set_input_to_controlled_actors` design is considered not great, but **reworking Input is out of scope**.
- ~~[proposed] For this refactoring: keep the current mechanism working for the root world, and expose the frame's input via `shared.input`.~~ [rejected] superseded in round 5.

[accepted] **Round 5 (author): remove `InputMaster` now, Godot-style input.**
- One shared `InputSystem` collects and prepares input once per tick.
- Any actor of any world reads it through `shared.input`: mouse, key states, and mainly **input actions** (the existing action system; actions stay hard-coded for now).
- `InputMaster` (`LocalMaster`/`RemoteMaster`), `ControlledActor::get_input_master` and `InputSystem::set_input_to_controlled_actors` are deleted. "Maybe we'll come back to something similar later if needed."
- Nested worlds get the same `EngineSharedSystems`, so the question "do nested worlds get input" disappears: an actor that needs input reads it, one that doesn't, doesn't.
- The 2D/3D demo player reacting to the keyboard inside a nested world isn't a problem; its logic will change later. The task now is the engine architecture, not the tutorial mechanics.

[proposed] Claude: no counter-arguments, and the code supports it:
- `RemoteMaster` is **not implemented**: `set_input_to_controlled_actors` hits `unimplemented!("Remote input aster didn't implement yet")` for it. Remote players are `PlayerDoll` actors driven by network messages, not by an input master.
- `LocalMaster` only caches a copy of `ActionsFrameState`, which `InputSystem::get_input()` already computes on demand. Actors call `shared.input.get_input()` (or a cached per-tick `shared.input.actions()`) instead.
- `InputSystem` stores its actions as `HashMap<SomeButton, Arc<Mutex<(ButtonActions, Action, i32)>>>`, although input is only touched on the main thread. With direct access the `Arc<Mutex<…>>` is unnecessary → plain values (the "remove locks that aren't needed" rule). [proposed] See §23.

---

## 9. Rendering

### 9.1 Author's initial idea (superseded)

Each `World` would get its own `Renderer` in its own tokio task, with configurable frequency and pause/resume, like the current render task.

### 9.2 Author's concern

The current renderer checks that previous GPU work finished before submitting (no queueing, §2.4). With several independent renderers, can each track only its own work, or would they wait on each other's?

### 9.3 Claude's analysis [accepted] (accepted)

- **Valid for the current mechanism.** `device.poll(Poll)` → `QueueEmpty` reports the state of *the whole queue*, so N renderers would see each other's work.
- **Per-submission tracking is possible in wgpu** (`SubmissionIndex` from `submit`, `queue.on_submitted_work_done`, `PollType::Wait { submission_index }`), so tracking isn't the real blocker.
- **The real reasons against independent renderers:**
  1. **One `Device` is mandatory**: parents sample children's textures, and resources can't cross devices.
  2. **One device = one queue, executed in order.** A heavy nested raymarch queued just before the main frame delays the player's frame, which is exactly the latency the author avoids.
  3. **Inter-world data dependency**: the child's texture must be rendered before the parent pass that samples it. Independent tasks give no ordering.
- **Conclusion [accepted]:** a single render loop with a shared **Render Scheduler** that encodes all due worlds into **one encoder and one `submit`** per frame. The `QueueEmpty` check stays valid.

### 9.4 Render Scheduler design [accepted] (concept) / [proposed] (details)

```
EngineSharedSystems.render: Option<RenderScheduler>
  main-thread side: device, queue, registration: mpsc::Sender<WorldSlot>, ...
  render loop (owns exclusively, no mutex): surface, upscale, UI renderer, worlds: Vec<WorldSlot>

WorldSlot { shared: Weak<WorldRenderShared>, depth: u32, frame_counter: u32 }

WorldRenderShared {                    // one Arc per world, held by the world's render system
    render_rate: AtomicU32,            // 0 = paused, 1 = every frame, 2 = every 2nd, 3 = every 3rd, ...
    output: AtomicU64,                 // packed: width u16 | height u16 | scale f32 bits (see §10)
    gpu: Mutex<WorldGpu>,              // pipelines, bind groups, output texture, ...
}
```

- **Data:** during `simulate()` the world writes its buffers via `queue.write_buffer` (thread-safe, no lock), as now.
- **Commands:** `WorldGpu::encode(&self, &mut encoder)` records the world's passes into its output texture. The render loop calls it. No prebuilt command buffers.
- **Frame:**

  ```
  drain registration channel (try_recv) → push slots
  if !queue_empty || too early → skip
  for slot in worlds, deepest first:
      upgrade() fails → remove slot (world dropped)
      due by render_rate? → gpu.try_lock() → encode (if locked: skip this world this frame)
  root: upscale its output texture to surface → UI → one submit → present
  ```

- **Deepest first:** a child's texture is ready before its parent samples it. Depth = parent depth + 1.
- A skipped frame keeps the last image in the texture.

#### 9.4.1 Round-4 simplifications [proposed] (proposed by Claude while drafting §21)

1. **No `Mutex<WorldGpu>` at all.** In wgpu 27 the resource types (`Buffer`, `Texture`, `BindGroup`, `RenderPipeline`, …) are cheap `Clone` handles. So:
   - the main-thread `WorldRenderSystem` keeps **clones of the world's buffers** and writes them via `queue.write_buffer` (thread-safe, as today)
   - the pipelines, bind groups and output texture are **moved into the `WorldSlot`** at registration and are then touched **only by the render thread**
   - the shared `Arc<WorldRenderShared>` then contains only atomics (`render_rate`, packed output size)

   Nothing on the main thread ever mutates `WorldGpu` after creation (resolution/scale go through the atomic; texture recreation and bind-group rebuilds happen on the render thread), so the mutex and the `try_lock` skip disappear. This follows the author's rule "remove every lock that can be removed without unnecessary complexity". Supersedes the `gpu: Mutex<WorldGpu>` / `try_lock` part of D10.
2. **No `depth` field: render in reverse registration order.** A nested world is always created by an actor of an already-existing (= already registered) world, so registration order is a valid parent-before-child order. Iterating the slots **in reverse** renders children before parents with no depth bookkeeping and no need to pass the parent's depth into `World::new`.
   - Edge case: if a `World` value is *moved* to an actor of a world registered later, the parent samples the child's previous frame (one frame of texture latency). Harmless.
3. **The presented world is explicit.** The engine tells the scheduler which world goes to the window (`present_world(Some(id))`). If none (startup, or the presented world is gone), the render loop draws the **splash texture** (§21.10) through the existing upscale pipeline.
4. Torn data between buffers: `simulate()` writes ~11 buffers one after another, and the render thread may submit between two writes. This is **exactly as today** (`send_data_to_renderer` doesn't lock the renderer either), so it's not a regression. Not addressed in this refactoring.

### 9.5 Registration via channel, auto-unregistration via `Weak` [accepted]

[accepted] Requirement: when a `World` is destroyed, it must be unregistered **automatically**, including worlds created and destroyed dynamically inside other worlds.

[accepted] Accepted:
- The world holds `Arc<WorldRenderShared>`; the scheduler holds `Weak`. When the world is dropped, `upgrade()` returns `None` and the slot is removed. No `Drop` impl is needed.
- **Registration goes through an `mpsc` channel.** The render loop owns `Vec<WorldSlot>` exclusively, so neither registration nor iteration needs a mutex. `Sender` is `Clone + Send`, so registration works from any thread, including an async world-build task.
- `try_lock()` on `WorldGpu`: the render loop never blocks; it skips a busy world for one frame.
- Author's principle [accepted]: **remove every lock that can be removed without unnecessary complexity.**

**How `Arc`/`Weak` work (explained to the author):**
- `Arc<T>` points to one heap block `{ strong: AtomicUsize, weak: AtomicUsize, data: T }`.
  - `clone` does `strong += 1` atomically; drop does `strong -= 1`.
  - `downgrade` does `weak += 1`.
- `Weak::upgrade()`:
  1. load `strong`
  2. if it is 0, return `None`
  3. otherwise CAS `n → n+1`; on success return `Some(Arc)`, and if another thread changed the counter meanwhile, retry
- Only atomics with Acquire/Release ordering: **no mutex, no syscalls, no thread blocking**. Lock-free (strictly, not wait-free: a CAS may retry under contention, which is negligible here).
- `T` is dropped on the thread that releases the last strong `Arc`; the memory is freed once the weak count is 0 too. If the world is dropped mid-frame, its `WorldGpu` is freed on the render thread after that frame. That is safe, because wgpu resources are refcounted and thread-safe to drop.

### 9.6 Dynamic `RenderRate` [accepted]

- [accepted] Changeable at runtime via `world.set_render_rate(n)`.
- [accepted] Encoding: `AtomicU32`; `0` paused, `1` every frame, `2` every 2nd, `3` every 3rd, …
- [accepted] Counted in **render frames** (Render Scheduler frames), not simulation ticks. Rendering and simulation are independent processes with possibly different rates. In the future, target simulation and render rates may be configured separately; the current coupling is accidental.
- [proposed] Optional: a "dirty" flag, so a world is rendered only if it was simulated since its last render.

**Atomics and `Relaxed` (explained to the author; the author's model was partly right):**
- An atomic operation is **indivisible**: a `load` sees either the old or the new value in full, never a half-written one. This is not a queue. The CPU guarantees it through cache coherence (the core gets exclusive ownership of the cache line for the write). On x86/ARM an aligned `u32` `load`/`store` with `Relaxed` compiles to an ordinary `mov`/`ldr`/`str`, with no extra cost.
- All threads agree on **one modification order per atomic variable** (coherence). Once a thread has seen value B, it will never later see the older value A of the same variable.
- `Relaxed` gives exactly that **and nothing more**: no ordering guarantee relative to *other* memory operations.
  - Example: if thread 1 writes data `X` and then a flag `F` with `Relaxed`, thread 2 may see the new `F` but still the old `X`. `Release` (store) / `Acquire` (load) fix that.
- "The new value becomes visible a bit later" is roughly right in effect. The memory model only promises visibility "in reasonable time"; in practice it's nanoseconds to microseconds, not frames.
- For a standalone value like `render_rate`, `Relaxed` is correct. The scheduler sees the new value on the same or the next frame.

### 9.7 Output texture and headless [proposed]

[accepted] Requirements:
- each world renders into its own texture, obtainable from the `World` and usable by actors (the screen actor)
- normal and headless logic as identical as possible: no GPU resources in headless mode, yet the same game logic

[proposed] Proposal:
- Actors never see GPU types. A `World` exposes `WorldTextureId(u64)`: `Copy`, valid in headless mode too, just never resolved.
- The screen actor puts the id into its `VisualElement`.
- The parent's render system resolves id → `TextureView` via the scheduler when building its bind group, with a 1×1 placeholder if missing (headless, destroyed, not yet rendered, being recreated).
- A texture "generation" counter tells the parent when to rebuild its bind group after a child's texture was recreated (resolution/scale change).

~~[open] **Q5: how the parent displays the texture.**~~ → done in round 4: **moved to a separate issue**, `issues/programming/in-world-4d-screens/`, and **out of scope** for this refactoring. The author wants a real 4D object with depth-correct occlusion (rasterized quad with 4D vertices + depth texture from the SDF pass), not a 2D overlay. Earlier options listed here were:
- an SDF "screen" object in the parent's raymarch shader (texture binding, UV projection, fixed slots, e.g. 4, with a placeholder)
- a 2D overlay via the UI renderer ([rejected] rejected in round 4: not an in-world object)

[accepted] What this refactoring must provide: a world renders into its own GPU texture, the texture is obtainable (via `WorldTextureId`) and usable by other GPU resources / pipelines. Displaying it is the other issue's job.

### 9.8 Render loop thread (done in round 4)

The render loop currently runs as a **tokio task** on the shared 2-worker runtime: a busy loop with ~3 ms sleeps that occupies a worker almost permanently.

[proposed] Proposal: run the Render Scheduler loop on a **dedicated OS thread** (`std::thread::spawn`) instead.
- It's an infinite timing loop, not async I/O, and gains nothing from tokio.
- It no longer competes with net or loading tasks for tokio workers.
- This also removes the root of the "heavy loading tasks occupy the render worker" concern (§12.5).

[accepted] Accepted by the author (round 4): the concept of independent rendering stays, a `std::thread` replaces the tokio task.

#### 9.8.1 Clean shutdown [accepted] requirement / [proposed] design

[accepted] Author's requirement: **the render thread must stop correctly when the engine shuts down**; it must not outlive the systems it uses. (With tokio, the `Runtime` partly managed task lifetimes; with a plain thread we must do it ourselves.)

Facts (answering the author's question):
- `std::thread::spawn` returns `std::thread::JoinHandle<T>`. **Dropping it does not stop the thread**: the thread is *detached* and keeps running until its closure returns or the **process exits** (when `main` returns, remaining threads are killed without running destructors).
- Rust has no way to kill a thread from outside. The thread must be asked to stop and must check that request itself; then `join()` waits for it.
- `join()` returns `Err` if the thread panicked; we log it.

[proposed] Design:
- The registration channel becomes the render loop's only inbox: `enum RenderLoopMessage { RegisterWorld(..), PresentWorld(..), ResizeSurface(..), Shutdown }`.
- The render loop drains it every iteration; on `Shutdown` **or** when the channel is disconnected (all senders dropped) it leaves the loop, drops its GPU objects (surface, slots, UI renderer) and returns.
- `impl Drop for RenderScheduler`: send `Shutdown`, then `join()` the thread. So the thread can never outlive `EngineSharedSystems`.
- **Drop order matters:**
  - `Engine` declares `root_world` **before** `shared`, so worlds (and their `Arc<WorldRenderShared>`) are dropped first, then the scheduler joins its thread.
  - The surface holds an `Arc<Window>` clone. The thread is joined **before** `Engine` drops its own `Arc<Window>`, so the window is destroyed on the main thread (macOS requires window destruction on the main thread).
- The render loop's sleep (~3 ms) bounds the shutdown latency.

---

## 10. Output resolution and scale [accepted] (revised)

[rejected] **Superseded:** multiplicative inheritance of scale factors along the nesting chain. It conflicted with per-world aspect ratios (raised by Claude) and was dropped by the author for simplicity.

[accepted] **Now:**
- On creation a `World` gets two **independent** parameters:
  - **output texture resolution** (arbitrary; not tied to the window or parent; the aspect ratio follows from it)
  - **output texture scale factor**
- No automatic inheritance from the parent.
- Both are **changeable at runtime**:
  - `world.set_output_resolution(..)`
  - `world.set_output_scale(..)`
  - possibly a combined `world.set_output_resolution_and_scale(..)`
  - (names are provisional)
- The root world's resolution = window resolution, updated on window resize via `set_output_resolution`.
- Use case: **LOD**. A screen actor lowers the nested world's resolution or scale as the player's camera moves away.

[proposed] Implementation:
- Store resolution + scale **packed in one `AtomicU64`** (`width: u16 | height: u16 | scale: f32 bits`) inside `WorldRenderShared`.
- A combined change is one atomic `store`, so the render loop always reads a **consistent** triple. With separate atomics it could read a new resolution with an old scale and recreate the texture twice.
- Max 65535 px per side is plenty.
- Texture pixel size = `resolution × scale`, computed in the render loop. When it changes, the render loop recreates the output texture (it owns encoding, so there are no cross-thread races) and bumps the texture generation.
- The `i`/`O` keys keep working as `set_output_scale` on the root world.

---

## 11. Headless and selectively disabled systems

[accepted] Requirements:
- Keep headless mode, possibly extend it per world (no sound, no render, …).
- Logic should be identical in normal and headless mode.

[proposed] Proposal (minimal):
- `HeadlessEngine` disappears: headless = `EngineSharedSystems.render == None`. Then every world has `render == None` and `WorldTextureId`s never resolve.
- Per world: `WorldConfig { render: Option<WorldRenderConfig /* resolution, scale, rate */>, ... }`.
- **Net:** no per-world net gating [accepted] (author: don't restrict functionality by root vs nested; separate transport from game logic). With the `SessionController` approach (§13.4), a nested world simply has no connection unless one of its actors creates one.
- **Audio/UI:** no per-world gating for now. Nested demo worlds use dedicated actors. Note that spatial audio in a nested world uses that world's coordinates, which is the owner's/actor's responsibility.

[accepted] **Out of scope:** the UI rewrite (UI elements owned by actors, probably Fyrox UI, input routing to UI) and the Input rewrite.

---

## 12. World creation, World Descriptor, async loading

### 12.1 Requirements [accepted]

- Creating a level currently compiles shaders and creates pipelines. That is slow and blocks.
- **`World` creation must support an async mode** (start, get a handle, keep working, take the `World` when ready) **and a sync mode**.
- Preloading must be possible, e.g. preparing the first level while in the main menu. The main menu will itself be a `World` whose actors can start loads.

### 12.2 Can a pre-created, never-simulated `World` be cached? (discussion)

Author's argument: a `World` created but not yet simulated is just data in memory, like a descriptor. What is the difference?

Claude's answer:
- The author is right that **handing over one pre-built, unsimulated world once** is not a problem in itself. That is a "take-once" handle.
- What doesn't work is **caching** in the resource-cache sense, i.e. returning the same thing to several requesters or reusing it after use:
  1. **Not shareable or clonable.** A `World` holds actors (`ActorWrapper`, not `Clone`), physics state and its **own** dynamic GPU buffers and output texture. Two worlds can't share them, since each writes its own state. A second copy requires a fresh construction anyway.
  2. **Construction has side effects that would happen at preload time.** The author wants all creation events inside `World::new()`:
     - creating actors
     - initial broadcasts
     - "world created" events
     - actor initialization

     Some of these touch **shared** systems: registering in the Render Scheduler, an actor starting a sound, creating a UI element, opening a network connection. If the world is built during preloading, these fire while the player is still in the menu, not when the world is actually used.
- So the clean split, which the author also leans towards, is:
  - **cached, immutable, shareable (`Arc`) descriptor** with the expensive prepared data (parsed level + compiled pipelines)
  - **`World::new(descriptor, ..)`**: cheap, contains *all* creation events, produces an independent instance. Many worlds can be created from one descriptor.

[accepted] **Decided (round 4):** cache a **`WorldDescriptor`**. The author fully agreed:
- `WorldDescriptor` holds the prepared data needed to create a world, including the parts that are slow to prepare: generated shaders and created render pipelines. It can be preloaded, prepared asynchronously and cached.
- `World::new()` is a relatively fast operation that creates a concrete instance from a prepared descriptor. It contains everything tied to creating *this instance*:
  - creating and initializing the world's own systems
  - creating actors
  - registering the world's rendering in the shared `RenderScheduler`
  - sending the initial broadcast messages to actors
  - other per-instance initialization
- Heavy preparation happens in advance; creating and starting the world is encapsulated in `World::new()`. "This is more logical and simpler."

Note: Consequence found in the code (round 4, see §20.1): today's `Level` holds **already-constructed actors** (`main_actor: Option<ActorWrapper>`, `actors: Option<Vec<ActorWrapper>>`) that `World::set_new_level` *takes* out of it. A descriptor shared via `Arc` and used for several `World::new` calls can't hand out the same actor objects twice. [proposed] So the descriptor stores the **actor descriptions** (the parsed JSON entries of the map), and `World::new` constructs the actors from them with the existing `parse_json_*` functions. Constructing actors from JSON is cheap; the slow part (shader generation, pipeline creation) stays in the descriptor.

### 12.3 Naming: `LevelDescriptor` vs `WorldDescriptor` [accepted] `WorldDescriptor` (round 4)

- The WIP has `level_descriptor.rs`. The author is undecided; `WorldDescriptor` matches the new `World` abstraction.
- [proposed] Claude: `WorldDescriptor`. It's exactly "everything needed to construct a `World`", including GPU pipelines, i.e. more than level data.
- The JSON files can remain "maps/levels".

### 12.4 Replacing the root world [accepted]

- Done at the `Engine` level, by **request** through the root world's `outer_world_mailbox`. The engine decides whether to comply.
- [accepted] The requesting world may prepare the next world/descriptor **asynchronously in advance**, then send the request **with the prepared data attached** (e.g. `Box`ed, to keep the enum small). The engine doesn't have to figure out what to load.
- This replaces today's in-world `LoadNewLevelSync/Async` / `PreloadNewLevelAsync` commands and `World.preloaded_levels`.

[proposed] Example: `OuterWorldMessage::ReplaceRootWorld(Box<…>)`. Since `World::new` is cheap and the heavy part is in the descriptor, the payload can be an `Arc<WorldDescriptor>`, and the engine calls `World::new` itself.

### 12.5 Where heavy work runs [open]

The author doesn't want to fix `spawn_blocking` as the final decision, and proposed a separate tokio runtime (dedicated worker) for loading as one option.

Claude's clarifications and options:
- **`spawn_blocking` does not run on tokio worker threads.** It runs on tokio's separate *blocking pool* (lazily spawned threads, default max 512, configurable via `max_blocking_threads`). So it already doesn't occupy the render worker. The issue only arises when blocking work is done *inside a normal async task*.
- Tokio can't pin tasks to specific workers of a multi-thread runtime.
- Options:
  1. `spawn_blocking` on the shared runtime
  2. a separate runtime for the Resource System
  3. a dedicated loader thread (or small pool) with a channel; loading is mostly blocking I/O + CPU (shader compilation), so async adds nothing
  4. if `fyrox-resource` is adopted, it brings its own `TaskPool` (separate threads)
- [proposed] Recommendation:
  - Fix the root cause by moving the render loop to its own OS thread (§9.8). Then the tokio runtime only serves net/async I/O.
  - Heavy loads go to either the blocking pool or the Resource System's own pool, decided in the Resource System issue.
  - Optionally limit concurrent heavy loads so shader compilation doesn't saturate all CPU cores and starve the render thread's CPU work.
- [proposed] Pipelines should target **one fixed output-texture format** for all worlds (instead of the surface format, as now). Then a descriptor's pipelines are valid regardless of whether the world is root or nested.
- [open] Should an async-created world register with the scheduler at `World::new` (paused until first simulate), or on first simulate? Since `World::new` is cheap and runs on the main thread, register in `World::new`.

---

## 13. NetSystem in this refactoring

### 13.1 Current state (analysis)

`src/engine/net.rs` (~2000 lines) mixes three things:
1. **Transport:** matchmaking over WebSocket (`tokio-tungstenite`), game server over WebRTC (`matchbox_socket`, channel 0 reliable / 1 unreliable), connection state machine, timeouts.
2. **Game-session logic:** `process_message` (~950 lines) decodes `ServerMessage` / `NetMessageToPlayer` / `RemoteMessage` into actor messages and commands:
   - spawn `PlayerDoll`
   - deliver `RemoteMessage` to actor ids
   - `SessionControllerMessage::{JoinedToSession, NewSessionStarted, SetScore, TeamWin}`
   - `NetCommand::SetServerTime`, `PeerConnected/Disconnected`
   - change of the main actor id
3. **UI:** "press P to play online", connection status (calls into `UISystem`).

Also:
- It holds a `PlayerSettings` clone and `it_is_2d_3d_example`.
- **Outgoing:** actors push `CommandType::NetCommand(NetCommand::Send{Boardcast,Direct}NetMessage{Reliable,Unreliable}(..) | SendMessageToServer(..))` into the messenger, and `execute_command` calls `NetSystem::send_*`. About 60 call sites in 9 files: `main_player.rs` (22), `player_for_2d_3d_example.rs` (12), `holegun.rs` and `obstaclesgun.rs` (6 each), `machinegun.rs`, `players_doll.rs`, `obstacle_course_free_movement_player.rs` (4 each), `shotgun.rs`, `flag.rs` (2 each).
- `send_*` sends **immediately**: `webrtc_socket.channel_mut(n).send(packet, server_id)` pushes into matchbox's channel, which matchbox's async task transmits.
- `SessionController` exists today (placed by the CTF map via `level.rs`) but holds no network logic; `NetSystem` sends `SessionControllerMessage`s to it.
- Server side: the game server relays `ClientMessage::{DirectMessageToPlayer, BoardcastMessageToPlayers}` to clients, which is **emulated peer-to-peer** over client-server, a legacy of the WebRTC/P2P choice (§13.2).

### 13.2 Author's context [accepted]

- WebRTC (via Matchbox) was chosen when a web version was considered. The logic is still P2P-style (clients address each other; the server relays).
- In practice connections often went through TURN anyway, so the architecture drifted to client-server with WebRTC as transport.
- Browser support is no longer a priority.
- **Long term** (separate issue, out of scope):
  - authoritative server (anti-cheat)
  - possibly UDP/TCP instead of WebRTC
  - actors like `SessionController` encapsulating session networking

### 13.3 Scope for this refactoring [accepted]

- Make `NetSystem` **mainly a transport abstraction**, whatever is underneath (WebRTC, UDP, TCP, …):
  - create connections, maintain them, report their state
  - send and receive through a specific connection
  - several **independent connections at once**, managed via **connection handles**
- Move game-session logic out of `NetSystem` into actors, e.g. `SessionController`.
- Keep the current game networking working; the server keeps its current relay logic.
- Example of independent connections: `SessionController` ↔ game server; `GameServerBrowser` ↔ server list; `StatisticsController` ↔ stats server; all at the same time.
- Don't mix this with the long-term rework.

[proposed] Minimal transport API:
- `connect(kind, url/config) -> ConnectionHandle`, where kind is WebSocket (matchmaking) or WebRTC (game server)
- `state(h)`
- `send(h, packet, reliability)`
- `receive(h) -> drain of packets`
- `close(h)`

`NetSystem` works with `Packet` (`Box<[u8]>`). The protocol types already have `to_packet()` / `from_packet()` (`ClientMessage`, `ServerMessage`), so serialization stays with the protocol types and their users. `NetSystem` stays protocol-agnostic without a new abstraction. (The author mentioned NetSystem converting high-level messages to bytes, possibly with compression. That can be added inside `send` later if wanted. [open])

### 13.4 Variant 1 vs Variant 2 for outgoing game messages [accepted] Variant 1 (round 4)

[accepted] **Decided (round 4): V1.** All game network messages go through `SessionController`. `MainPlayer` and other actors that send game data pass it to `SessionController` as ordinary in-world messages; `SessionController` talks to `NetSystem` via `EngineSharedSystems` and sends to the game server. New message variants are added to the existing enums (between `MainPlayer` and `SessionController`, and for other actors that currently send to the network directly). The author considers this a fairly straightforward change.

The author described two options:
- **V1:** actors (`MainPlayer`, weapons, …) send ordinary in-world messages to `SessionController`, which sends them via its connection handle. Only `SessionController` uses the handle.
- **V2:** `SessionController` creates the connection and hands the `ConnectionHandle` to other actors, which send directly via `shared.net`.

Claude's analysis:
- **Incoming handling is identical in both variants, and it's the big part.** The ~950-line decoder `process_message`, plus connection/matchmaking flow, UI messages and server time, moves into `SessionController` (or a module it uses). The variants differ only in the ~60 outgoing call sites.
- **Latency is the same.** In V1 the message reaches `SessionController` in the same message pump within the same `simulate()`; `send_messages_and_process_commands` loops until all buffers are empty. It is then sent immediately (or flushed in `post_simulate`).
- **Nested worlds:** both isolate naturally. In V1, a `MainPlayer` in a nested world sends to *its* world's `SessionController`; if there is none, the direct message is dropped (`send_direct_messages` ignores unknown ids). In V2 it has no handle. No root/nested special-casing, as the author wants [accepted].
- **Long-term fit:** V1 is better. All protocol knowledge (which `ClientMessage`/`NetMessageToPlayer` to build, reliability) is concentrated in one actor. Moving to an authoritative server then changes `SessionController` + server, not 9 actor files. V2 keeps protocol construction spread across actors, which entrenches today's P2P style.
- **Effort now:** about the same.
  - V1: replace `messenger.send_command(NetCommand(...))` with a direct message to the session controller carrying the same payload (e.g. `SessionControllerMessage::Net(NetCommand)`). That is mechanical; the payload types stay the same.
  - V2: replace them with `shared.net.send(handle, ...)` and distribute the handle (another mechanism).
  - V1 needs actors to know the `SessionController`'s `ActorID`. Simplest: `SessionController` broadcasts `IAmSessionController(id)` on world start or when the connection is established, and interested actors store it. That is the same discovery style the code already uses (e.g. `GiveMeFlagsPostitions`). Fallback: broadcast the outgoing message (no id needed, slightly wasteful).
- [proposed] Recommendation: **V1.** → [accepted] accepted.

[proposed] Other things the session logic does today that must move with it (found in round 4, see §20.2):
- `NetCommand::NetSystemIsConnectedAndGetNewPeerID(new_id)` makes `execute_command` call `world.change_actor_id(main_actor_id, new_id)`: **the main player's actor id becomes its network peer id.** ~~`SessionController` needs a world command for that: `CommandType::ChangeActorId { old, new }`.~~ [rejected] round 5 → [accepted] the author's design: `SessionController` sends `MainPlayer` an ordinary message `ChangeYourId(new_id)`; `MainPlayer` itself sends `CommandType::ChangeMyId(new_id)`. An actor may change **only its own** id; the old id is taken from the command's `sender` (`Command` already has `sender: ActorID`).
- `PeerConnected` / `PeerDisconnected` spawn / remove `PlayerDoll`s → ordinary `SpawnActor` / `RemoveActor` commands from `SessionController`.
- `SetServerTime` → ~~`shared.time.set_server_time(..)`~~ stored inside `SessionController` (round 5).
- `RemoveAllHolesAndEffects` (sent from `net.rs` on new session / team win) → stays a world command, now sent by `SessionController`.
- `ShowConnectionStatusUI` (sent by `MainPlayer`) → becomes a message to `SessionController`, which owns the connection UI.
- `it_is_2d_3d_example` (held by `NetSystem` today) → read from the world's descriptor by `SessionController`.

### 13.5 Post-simulate net step [accepted]

- [accepted] After the root world's `simulate()`, `NetSystem` gets another tick so outgoing messages are sent immediately (minimal input-to-network latency).
- Shared-system updates are therefore not limited to "before `simulate`".
- [proposed] Note: today `send_*` already transmits immediately. If `NetSystem` keeps immediate sends, `post_simulate` costs nothing and is a natural place for future batching/packing or explicit flushes (UDP). If sends become buffered, `post_simulate` must flush.

---

## 14. Settings: `PlayerSettings` → constants + `GlobalSettings` [accepted] (direction) / [proposed] (details)

### 14.1 Author's decision [accepted]

- Remove `PlayerSettings`.
- Gameplay parameters of `MainPlayer` become **Rust constants in the actor code**.
- Real app-wide settings (mouse sensitivity, server connection data, scale factor, …) go into one global settings struct **inside `EngineSharedSystems`**, accessible to all actors.
- The name is provisionally `GlobalSettings` (preferred) or `GlobalConfig`.

### 14.2 Analysis of `PlayerSettings` fields

Today values come from `settings.json` (user file, optional), with fallback per key to the embedded `settings2.json`. So **a local `settings.json` can currently override gameplay physics** (gravity, speeds, damage), which is also a cheat vector that the change closes.

| Field | Used by | Proposed destination |
|---|---|---|
| `collider_radius`, `max_speed`, `max_accel`, `air_speed_mult`, `jump_y_speed`, `jump_w_speed`, `gravity_y_speed`, `gravity_w_speed`, `friction_on_air` | main_player, player_inner_state, **players_doll** (max_speed, max_accel, friction, radius), observer, obstacle-course player, 2d/3d player, level.rs | **constants** |
| `min_respawn_timer`, `max_respawn_timer`, `scanner_show_enemies_time`, `scanner_reloading_time` | main_player | **constants** |
| `energy_gun_hole_size_mult`, `energy_gun_add_force_mult`, `energy_gun_damage_mult`, `energy_gun_restoring_speed`, `machinegun_damage`, `machinegun_add_force`, `machinegun_heat_add_on_shot`, `machinegun_cooling_speed` | main_player / devices | **constants** (in the respective device modules if only devices use them) |
| `jetpak_w_speed`, `w_jump_time_reloading`, `rotation_along_w_standard_method` | parsed, **unused** | delete |
| `shadows_enable` | only commented-out code; render has its own `P` toggle | done in round 4: `GlobalSettings.shadows_enabled` with a default value; the `P` toggle changes it and it is saved |
| `mouse_sensivity: Arc<Mutex<f32>>` | input/players (~21 uses), `Settings` (+/− keys) | `GlobalSettings.mouse_sensitivity: f32`. A plain `f32` suffices once everyone reads it via `&mut shared`; the `Arc<Mutex>` existed only because every actor held its own `PlayerSettings` clone. |
| `matchmaking_server_url`, `bash_and_turn_servers`, `turn_server_username`, `turn_server_credential` | `NetSystem` | `GlobalSettings` (read by `SessionController` when creating connections) |
| `screen_resolution_scale` | parsed, **currently unused** (the renderer gets a hard-coded `1.0`) | `GlobalSettings`; use it as the root world's initial output scale |

[proposed] Constants placement:
- Put them in `main_player.rs`, where `PLAYER_MAX_HP`, `TIME_TO_DIE_SLOWLY`, `W_SCANNER_*` etc. already live, or in a `main_player/constants.rs`.
- Other actors (`players_doll`, `observer`, …) import them.
- Actors that need different values (obstacle-course player, 2d/3d player) can define their own.

[proposed] → (done in round 4, accepted implicitly; the author used `GlobalSettings` throughout) Naming: the WIP had `GameConfig` (the renamed `PlayerSettings`, loaded data) wrapped by `GlobalSettings` (renamed `Settings`). After this change there is only **one** struct, so **`GlobalSettings`** (the author's preference, already in the WIP) is the right name. `GameConfig` is dropped: it would also be confusing next to gameplay constants. The loader becomes `GlobalSettings::load()`, reading `settings.json` with the same path fallbacks; `settings2.json` disappears. (Superseded by §14.3 for the file locations.)

### 14.3 Persistent user settings file (done in round 4) / [proposed] details

[accepted] Author's concept:
- `GlobalSettings` are stored in a **local config file on the user's computer**.
- On first launch the app creates a Slice 4D folder in the user's home directory, e.g. `~/.slice4d/`, with a settings file inside, e.g. `~/.slice4d/GlobalSettings.json`. Names are provisional.
- If the file is missing, it is created with default values. Later launches read it.
- Settings changed at runtime (now via hotkeys, e.g. `P` for shadows; later via the menu GUI) are **saved to that file**, so the next launch uses the saved value, not the default.
- [accepted] **Lookup order** (the author changed his mind a few times and settled on this):
  1. a config file **next to the executable**
  2. otherwise the **user config file** (`~/.slice4d/GlobalSettings.json`)
  3. otherwise create the user directory and file with **defaults**

[proposed] Claude's notes and proposals:
1. **"Next to the executable" ≠ current directory.** Today `PlayerSettings` opens `./settings.json`, which is relative to the **current working directory**, not the binary. To really mean "next to the binary", use `std::env::current_exe()?.parent()`.
2. **Where to save changes.** Proposal: save back to **the file the settings were loaded from**. If that's the file next to the binary and it isn't writable (e.g. Windows installs into `Program Files`), log a warning and keep the change in memory only. The file next to the binary is then mainly a dev / server / portable override.
3. **Connection data should not be persisted in the user file** (important). `matchmaking_server_url`, ICE/TURN server list and TURN credentials are deployment data. If they're written into `~/.slice4d/…` on first launch, a later release that changes the server URL or rotates TURN credentials **won't reach existing players**: their saved file keeps the old values forever. Proposal: split `GlobalSettings` into
   - **user preferences** (sensitivity, volume, resolution scale, shadows, …): loaded/saved via the lookup order above
   - **connection config**: compiled-in defaults (today's embedded `settings.json`), optionally overridden **only** by the file next to the binary, never saved to the user file

   Both still live in one `GlobalSettings` struct (as nested structs), so there's no new system, only two fields. [open] (§18)
4. **Directory.** `~/.slice4d/` is simple and works everywhere. The platform convention would be the OS config dir (`~/.config/slice4d/` on Linux, `%APPDATA%\Slice4D\` on Windows, `~/Library/Application Support/Slice4D/` on macOS), via the small `dirs` crate. Claude has a slight preference for the platform config dir on Windows (users rarely look into `C:\Users\name\.slice4d`), but either is fine. [open] (§18)
5. **Forward compatibility.** Missing keys must fall back to defaults (a new release adds a field → the old file still loads), unknown keys ignored. With `serde` derive this is `#[serde(default)]` on the struct. Today the project parses JSON manually with `serde_json::Value`; adding `serde` with the `derive` feature (already in the dependency tree via other crates) is the simplest way to get both load and save. [proposed]
6. **Also persist:** `master_volume` (the `N`/`M` hotkeys change `AudioSystem.master_volume`, currently not saved) and the root world's output scale (`i`/`O` keys → `screen_resolution_scale`). [proposed]
7. **When to save:** immediately on change is simplest (these changes are rare: a key press or a menu click). For keys that change a value continuously while held (`J`/`K` sensitivity, `N`/`M` volume), save on key release, or mark dirty and save at most once per second / on exit. [proposed] Proposal: `GlobalSettings::save()` called on key release and on engine shutdown.
8. **Name of the file/dir:** follow the struct name → `global_settings.json` (snake_case like other project files, e.g. `matchmaking-server-config.json` uses kebab-case; either is fine). [open]

---

## 15. Proposed staging (history; [rejected] superseded by the final plan in §24.4)

1. **Structure:**
   - `EngineSharedSystems` + one `World`
   - renames (`EngineHandle→Messenger`, `World→Actors`)
   - message pump into `World`
   - `World::simulate()`
   - main loop split into pre / simulate / post
   - `GlobalSettings` + constants (`PlayerSettings` removed), settings file lookup/save (§14.3)
   - `GlobalTimeSystem` (from today's `TimeSystem`)
   - delete the WIP files
2. **Actor signature:** `WorldContext` + `&mut EngineSharedSystems`; `WorldTimeSystem` (world delta); `EffectsSystem` dissolved; `InputMaster` removed (actors read `shared.input`); `SpawnControlledActor` removed; `ActorWrapper` impl generated by a macro (§23.2); message pump inside `Messenger`.
3. **NetSystem as transport:**
   - connection handles
   - session logic, `process_message` and connection UI moved into `SessionController`
   - outgoing via V1 [accepted]
4. **Render split:**
   - `RenderScheduler` on its own OS thread with clean shutdown (inbox channel, `Weak` slots, atomics)
   - per-world `WorldRenderSystem` / `WorldRenderShared` / `WorldGpu`; fixed output format
   - splash texture when no world is presented
   - root world only; behaviour identical
5. **Nested worlds:**
   - creation by actors
   - `outer_world_mailbox`
   - `set_render_rate`, `set_output_resolution/scale`
   - depth ordering, `WorldTextureId`
   - headless parity
   - root-world replacement via mailbox
6. **WorldDescriptor + async world creation** (minimal: `WorldDescriptor::load` on `spawn_blocking`; the Resource System comes later). Initial world via `EngineConfig`.
7. ~~**Showing a nested world's texture** (Q5) and the tutorial screen.~~ → moved out of this refactoring (round 4): `issues/programming/in-world-4d-screens/`.

[accepted] Expected result (round 4, author): **the existing game works exactly as now** (CTF map, actors, physics, controls, rendering, networking, all mechanics) on the new architecture, plus the technical ability to create nested worlds, simulate them independently, render them into separate textures and control their render rate.

[proposed] Stages 4 and 6 overlap (the descriptor carries the pipelines the render split needs); in practice `Level` → `WorldDescriptor` may happen together with stage 1 or 4. The order will be fixed in the final spec.

---

## 16. Out of scope (tracked elsewhere or later)

- Resource System → `issues/programming/resource-system/`
- Authoritative server, dropping emulated P2P, UDP/TCP transport → `issues/programming/network-authoritative-server-and-transport/`
- UI rewrite (Fyrox UI, actor-owned UI elements)
- Input rewrite (`InputMaster`)
- Displaying textures on in-world 4D screens → `issues/programming/in-world-4d-screens/`

---

## 17. Decision log

| # | Topic | Decision | Status |
|---|---|---|---|
| D1 | Purpose | Nested worlds for the tutorial screen (2D/3D analogy) | Accepted |
| D2 | System split | Shared (Audio, UI, Input, Net, Time, GlobalSettings, Render Scheduler, Resources) vs World (Physics, Render data, Messenger, Actors) | Accepted |
| D3 | Naming | Keep the WIP names; `simulate()` (not `tick`) | Accepted |
| D4 | WIP files | Claude may reuse or delete them (no approval needed); plan: implement from live code, delete in stage 1 | Accepted |
| D5 | Ownership | Nested worlds are owned by actors; the engine doesn't track worlds | Accepted |
| D6 | Simulation | The owner drives `simulate(shared)`; no simulation scheduler | Accepted |
| D7 | Render vs sim | Independent; render data updated in `simulate`, drawn by the scheduler | Accepted |
| D8 | Renderer | One render loop + Render Scheduler; one encoder/submit per frame | Accepted |
| D9 | Unregistration | `Arc` (world) / `Weak` (scheduler), automatic | Accepted |
| D10 | Registration | `mpsc` channel; the render loop owns the slots; no mutex; `try_lock` on `WorldGpu` | (done in round 4 [proposed]: drop the `WorldGpu` mutex entirely, §9.4.1) |
| D11 | RenderRate | `AtomicU32` divisor (0 paused, N every Nth), runtime-changeable, counted in render frames | Accepted |
| D12 | Messaging | In `Messenger`; `outer_world_mailbox`; inward via existing buffers with sender id 0; hand-written enums | Accepted |
| D13 | Root replacement | Requested via the root world's mailbox; the engine decides; the payload may carry prepared data | Accepted |
| D14 | Input | Shared, no restrictions; Input rewrite out of scope | Accepted |
| D15 | Output resolution/scale | Independent per world, runtime-changeable; **no inheritance** (supersedes the earlier idea) | Accepted |
| D16 | Resolution+scale storage | Packed `AtomicU64` for consistent updates | Proposed |
| D17 | Texture abstraction | `WorldTextureId` + placeholder + generation counter | Proposed |
| D18 | Headless | `render: None` at engine level; no per-world net gating | [accepted] / [proposed] |
| D19 | Resource System | Separate issue | Accepted |
| D20 | World caching | Cache an immutable `WorldDescriptor` (with pipelines); `World::new` is cheap and holds all creation events | [accepted] direction / [proposed] details |
| D21 | Descriptor name | `WorldDescriptor` | done in round 4 |
| D22 | NetSystem | Transport abstraction with connection handles, multiple connections; session logic → actors | Accepted |
| D23 | Outgoing game messages | V1: all game traffic through `SessionController` | done in round 4 |
| D24 | Post-simulate net step | Yes | Accepted |
| D25 | Settings | `PlayerSettings` removed; gameplay → constants; app settings → `GlobalSettings` in shared systems | Accepted |
| D26 | Settings name | `GlobalSettings` (`GameConfig` dropped) | done in round 4 |
| D27 | Time | `GlobalTimeSystem` (shared) + `WorldTimeSystem` (per world, own delta) | done in round 4 |
| D28 | Render thread | Dedicated OS thread instead of a tokio task; clean shutdown (§9.8.1) | done in round 4 |
| D29 | Heavy loading | Not fixed to `spawn_blocking`; options listed in §12.5 | Open |
| D30 | Agent docs | `AGENTS.md` main, `CLAUDE.md` → `@AGENTS.md`; `issues/YYYY-MM-DD-name.md` | [accepted] done |
| D31 | In-world screens (Q5) | Separate issue `issues/programming/in-world-4d-screens/`; out of scope here; the refactoring only provides a usable per-world texture | done in round 4 |
| D32 | Shadows setting | `GlobalSettings.shadows_enabled` with a default, saved when toggled | done in round 4 |
| D33 | Settings persistence | User settings file; lookup: next to the binary → user dir → create with defaults; runtime changes are saved | done in round 4 (details [proposed] §14.3) |
| D34 | Initial world | Passed to `Engine::new`; now the CTF map, later the main menu | done in round 4 (author) |
| D35 | Loading placeholder | Splash texture drawn by the render loop when no world is presented; not a `World` | [proposed] (§21.10) |
| D36 | Issues format | Each issue keeps a Discussion section with intermediate ideas; superseded ideas are marked, not deleted | done in round 4 |
| D37 | Expected result | The game works exactly as before, on the new architecture; nested worlds can be created, simulated, rendered to textures, with controllable render rate | done in round 4 |
| D38 | Descriptor actors | `WorldDescriptor` stores actor descriptions (map JSON entries); `World::new` constructs actors | [proposed] (§12.2, §20.1) |
| D39 | Render order | Reverse registration order instead of a depth field | [proposed] (§9.4.1) |
| D40 | Outer-world messages | `OuterWorldMessage` with standard variants + one `Specific(..)` variant for game-specific messages | done in round 5 |
| D41 | Actor creation | `WorldDescriptor::create_actors()` builds new actors from stored JSON (existing `parse_json_*`); `World` doesn't know about JSON (supersedes the "World parses" part of D38) | done in round 5 |
| D42 | Actor ids | An actor changes only its own id: `ChangeYourId(new)` message → `CommandType::ChangeMyId(new)`, old id = `command.sender` | done in round 5 |
| D43 | Messenger | All message delivery, distribution and command execution live in `Messenger`; `World` only calls it | done in round 5 |
| D44 | Session time | Server/session time and sync data in `SessionController`, not in `GlobalTimeSystem`; it stamps outgoing messages | done in round 5 |
| D45 | Input | `InputMaster` and `set_input_to_controlled_actors` removed; actors read `shared.input` (Godot-style), actions stay hard-coded | done in round 5 |
| D46 | World ↔ descriptor | `World` doesn't store its descriptor; needed values are copied into its systems; `World::new(&WorldDescriptor, ..)` | done in round 5 |
| D47 | GPU context | `GpuContext` (device, queue, output format, layouts; Clone) as a **separate field `shared.gpu: Option<GpuContext>`** | done in round 6 (author; supersedes "inside `RenderScheduler`") |
| D48 | Splash | Render-loop `SplashScreen` component with time input; static logo first, animated indicator possible later | done in round 5 (direction) |
| D49 | `pending_root_world` | Keep (async initial load) because a blocked main thread makes the OS mark the window "Not Responding" | done in round 6 |
| D50 | Naming | `on_level_loaded` → `on_world_started`, `boardcast` → `broadcast`, `WorldRenderSystem` | done in round 5 |
| D51 | Settings split | User preferences in the user file; connection data compiled in, overridable only next to the binary | done in round 5 |
| D52 | Settings location | Platform config dirs (`~/.config/slice4d`, `%APPDATA%\Slice4D`, `~/Library/Application Support/Slice4D`) | done in round 5 |
| D53 | `serde` | `serde` derive for settings load/save | done in round 5 |
| D54 | Extra cleanups | Round 6 outcome: [accepted] `macro_rules!` for `ActorWrapper`; [rejected] keep `SpawnControlledActor`; [accepted] `InputSystem` without `Arc<Mutex>` (if no complications); [rejected] keep LIFO `Vec::pop()`; [accepted] `EffectsSystem` → function; [rejected] keep `async fn Engine::new` (wgpu) | done in round 6 |
| D55 | Settings file names | `user_settings.json`, `connection_settings.json` | done in round 6 |
| D56 | UI renderer | Re-bound to the presented world's buffers (HUD scanner reads world data) | done in round 6 (found in code, part of the spec) |
| D57 | Network scope | Network only in worlds with a `SessionController` (today: CTF map) | done in round 6 (consequence of V1, accepted behaviour change, §24.2) |
| D58 | Issue structure | Part I concept/rationale, Part II discussion & history, Part III final architecture, Part IV spec + implementation log | done in round 6 |
| D59 | Start | The author authorized starting the implementation right after the final spec, without another confirmation | done in round 6 |

---

## 18. Open questions

All design questions are resolved (round 6). Resolved in round 4: Q-delta, Q5, Q-net-variant, Q-descriptor-name, Q-render-thread, Q-shadows-default. Round 5: Q-settings-split, Q-settings-dir, Q-review-21, naming. Round 6: Q-pending-root (keep), Q-gpu-context (separate field), Q-cleanups (D54), Q-settings-files (D55), Q-net-serialization (raw `Packet`s, as proposed), Q-heavy-work (`spawn_blocking` for now; final answer in the Resource System issue).

Questions that come up during implementation are recorded in §24.9.

---

## 19. Discussion (chronological)

1. **2026-11-11. Initial analysis.** Claude analyzed the repo, wrote the initial `CLAUDE.md`, and noticed the WIP refactor and its renames.
2. **2026-11-11. Author's brief.** The author described the project, the goal (tutorial screen → nested worlds), and the target architecture:
   - Shared Systems vs World
   - nesting and simple inter-world messaging
   - selective headless and a texture abstraction
   - rendering options: independent renderers vs Render Scheduler, the GPU-tracking concern, relative frequency, auto-unregistration
3. **2026-11-11. Claude's first analysis.**
   - Scheduler confirmed; the real blockers are a single queue, a single device and ordering.
   - Proposed `Weak` registration, divisor rate, depth ordering, `WorldTextureId`, outbox, context param, dissolving Effects, staging.
   - Asked Q1–Q8.
4. **2026-11-12. Author's answers (round 2).**
   - Claude decides about the WIP files; keep the names.
   - `World::simulate()` driven by owner actors; render separate.
   - `Weak` accepted, with a question about its synchronization.
   - Runtime-changeable `RenderRate`.
   - Input shared and unrestricted; UI/Input rewrites out of scope.
   - Messaging in `Messenger`.
   - Inherited scale factor (later dropped).
   - Resource System concept; async world creation; "World as a resource?".
   - `AGENTS.md` main; create this issue.
5. **2026-11-12. Claude (round 2).** Created `AGENTS.md`/`CLAUDE.md`/this issue. Explained `Arc`/`Weak`. Proposed the channel registration, the `AtomicU32` rate, the outer mailbox, `JoinHandle` vs own handle, the `fyrox-resource` evaluation and the descriptor-vs-world split. Asked new questions.
6. **2026-11-12. Author's answers (round 3).**
   - Delta lives in `TimeSystem` (questioned the need for accumulated time).
   - Channel, `try_lock` and atomics accepted; asked how atomics and `Relaxed` work.
   - Messenger design accepted, inward sender id 0.
   - Scale inheritance **dropped**: independent resolution + scale, runtime-changeable.
   - Resource System → separate high-level issue; prefers `fyrox-resource` if `Reflect`/`Visit` aren't a blocker.
   - Doesn't want `spawn_blocking` fixed; suggested a separate runtime.
   - Questioned why a pre-built world can't be cached; leans to a cached `WorldDescriptor` with all creation events in `World::new`.
   - **Net:** NetSystem becomes a transport with handles and multiple connections; session logic moves to actors; V1 vs V2 to evaluate; mandatory post-simulate net tick; no root/nested net gating; long-term authoritative server and UDP/TCP are a separate future task.
   - **Root world replacement** via mailbox request, possibly with prepared data.
   - RenderRate counted in render frames; Q5 deferred.
   - WIP files may be deleted.
   - **Settings:** `PlayerSettings` → constants + `GlobalSettings` in shared systems.
7. **2026-11-12. Claude (round 3).**
   - Updated this issue; created the Resource System and future-network issues.
   - Explained atomics/`Relaxed` and the nested-delta subtlety.
   - Explained why world *instances* aren't cacheable (not shareable; construction side effects at preload time).
   - Clarified that `spawn_blocking` uses a separate pool and proposed a dedicated render thread.
   - Analyzed the network code and recommended V1.
   - Analyzed the `PlayerSettings` fields; proposed the `GlobalSettings` name and a packed `AtomicU64` for resolution+scale.
8. **2026-11-12. Author's answers (round 4).** "We are close to finishing the architecture discussion."
   - **Time:** agreed with the nested-delta problem ("I hadn't fully thought it through"). Proposed `WorldTimeSystem` per world (own delta since its previous `simulate()`) and `GlobalTimeSystem` in shared systems (tick count, app/session start time, date/time, other general info; maybe not needed much now but useful long-term). Assumed the current `TimeSystem` mostly becomes `WorldTimeSystem`; exact split after studying the code. → §5.3
   - **`WorldDescriptor`** approved as the name, and the split approved: heavy prepared data (generated shaders, pipelines) in the descriptor, preloadable/async/cacheable; `World::new()` fast, containing per-instance creation (own systems, actors, registration in `RenderScheduler`, initial broadcasts). → §12
   - **Net V1** approved: all game messages through `SessionController`; new message variants in the existing enums; "a fairly straightforward change". → §13.4
   - **Render thread** on a dedicated OS thread approved; asked to verify whether dropping `std::thread::JoinHandle` stops the thread, and required a clean shutdown of the render thread when the engine stops. → §9.8.1
   - **Shadows** stay in `GlobalSettings` with a default. **Settings persistence**: a user config file (e.g. `~/.slice4d/GlobalSettings.json`), created with defaults if missing; runtime changes (hotkeys now, menu GUI later) saved; lookup order next-to-binary → user file → create defaults; names open. → §14.3
   - **Q5 / in-world screens:** a 2D overlay doesn't fit; the screen should be a real 4D object: depth texture from the SDF pass, then a separate pass of two-triangle quads with 4D vertices and a 4D transform in the vertex shader, UVs, static or world texture, depth test against the SDF scene; W-thickness with two variants (constant size within a range vs double-pyramid size variation); arbitrary 4D rotation is an open problem. **Not in this refactoring** → separate issue `issues/programming/in-world-4d-screens/`. The refactoring only has to provide a per-world GPU texture usable by other pipelines.
   - **Issues format:** each issue gets a Discussion section with intermediate ideas, arguments and variants; superseded ideas are marked, not deleted; "we don't save bytes".
   - **Final check requested:** analyze the whole discussion, the code and this issue; raise remaining questions or contradictions; then present the **target architecture as Rust structures** (fields, types, ownership, main methods) and **examples of key operations** (create `Engine`, load/prepare `WorldDescriptor`, `World::new`, `simulate`, nested simulate from an actor tick, render registration, `RenderRate`, resolution/scale, getting the world texture, world↔owner messages, root replacement, shutdown), plus an **`OuterWorldMessage` enum** proposal (standard requests: quit, replace root world, destroy this world; plus game-specific messages, without over-complicating).
   - **Initial world:** set when creating the `Engine` (e.g. an `Engine::new` argument); now the CTF map, later the main menu. Wants a simple **loading placeholder** (logo/static texture) while the first world's descriptor/pipelines load; unsure whether that should be a full `World` or a simpler engine state; asked for the simplest option that allows replacing the initial map with a menu later and showing a splash during long loads.
   - **Expected result of the refactoring:** the existing game works exactly as now (CTF map, actors, physics, controls, rendering, networking, all mechanics), but on the new architecture (`EngineSharedSystems`, `World`, `Actors`, `Messenger`, `WorldDescriptor`, `RenderScheduler`, …), with the technical ability to create nested worlds, simulate them independently, render them to separate textures and control render rate. Screens in 4D come later.
   - Process: after the author reviews §21, Claude does a final update of this issue into the implementation spec; implementation only after a separate confirmation. Code must not be changed yet.
9. **2026-11-12. Claude (round 4).**
   - Re-read the engine code against the design and listed the problems the design must handle (§20).
   - Corrected the time split: most of today's `TimeSystem` is engine-loop timing → `GlobalTimeSystem`; `WorldTimeSystem` is small and new (§5.3).
   - Answered the `JoinHandle` question (dropping it detaches; the thread must be asked to stop and joined) and designed shutdown via a `Shutdown` message + `join()` in `Drop`, with the drop-order constraints (§9.8.1).
   - Proposed two simplifications of the render design: no `Mutex<WorldGpu>` (GPU objects move into the slot; wgpu 27 handles are `Clone`), and reverse registration order instead of depth (§9.4.1).
   - Settings: pointed out that "next to the binary" must use `current_exe()`, that persisting connection data in the user file would freeze server URLs/TURN credentials for existing players, and proposed `serde` derive with defaults (§14.3).
   - Proposed a splash texture in the render loop instead of a loading `World` (§21.10).
   - Created `issues/programming/in-world-4d-screens/`.
   - Wrote the Rust draft (§21) and key operations (§22).

10. **2026-11-12. Author's review (round 5).** "Overall it looks fine", with remarks:
    - **Actors from the descriptor:** agreed to store actor descriptions instead of requiring `Clone`; keep `parse_json_*` for now, but **not** in `World`: `World::new` calls something like `descriptor.get_actors()`, and the descriptor builds new instances. `World` must not know how actors are made from JSON. → D41
    - **Actor ids:** doesn't like actors changing other actors' ids. `SessionController` sends `ChangeYourId(new_id)` to `MainPlayer`, which sends `ChangeMyId(new_id)` to the `Messenger`; the old id comes from the sending actor's context. → D42
    - **Messenger:** (not mentioned before) all logic of passing and processing messages between actors must be inside `Messenger`, which may access `Actors` and other world systems; `World` only calls its methods. → D43
    - **Session time:** must not move to `GlobalTimeSystem`. Everything about a concrete game session (server time, timestamps, offsets, sync parameters) belongs to `SessionController`; there could be several sessions; today's placement is a legacy of the raw architecture. → D44
    - **Input:** `InputMaster` is outdated; remove it now (maybe something similar later). Godot-style: one `InputSystem`, every actor of every world reads `shared_systems.input` (mouse, keys, mainly input actions, still hard-coded). The 2D/3D demo player reacting to the keyboard is not a problem; its logic changes later. Asked for counter-arguments if any. → D45
    - **Output format:** fixed format for worlds, keep sRGB so colours stay the same. **FinalTrigger:** a test actor; described the target flow with the Resource System (request descriptor at creation → cached or async handle → `ReplaceRootWorld` with the prepared descriptor; no engine-level preload/load commands); for now the minimum that keeps current behaviour. → §20.10
    - **Shutdown:** agreed (send stop, `join()`, correct lifecycle of the thread and GPU resources). **Time split:** agreed, with session time in `SessionController`.
    - **Splash:** agreed not to use a `World`, but wants some motion later (logo + rotating loading indicator) so the user sees the app isn't frozen; maybe the Render Scheduler shows it itself; for now the simpler variant, keeping animation possible. → D48
    - **`Mutex<WorldGpu>` removal:** agreed.
    - **`pending_root_world`:** unsure it's needed: the render thread could show the splash while `Engine::new` loads the first descriptor and creates the first world synchronously. Doesn't insist; asked Claude to re-evaluate. → §21.10
    - **GPU context:** not visible in `EngineSharedSystems`; where are `Device`, `Queue` etc. for creating descriptors and world render systems? Separate field, or inside the scheduler with an explicit interface? → §21.7
    - **`World.descriptor`:** why keep it? The descriptor is only needed in `World::new`; an actor that wants to recreate the world can hold the descriptor resource itself. Remove unless there's a reason. → D46
    - **`OuterWorldMessage`:** fine; extend as needed. **Naming:** `on_world_started`, `broadcast`, `WorldRenderSystem` agreed.
    - **Settings:** connection data compiled in and overridable only next to the binary — approved. Platform config directories — approved. `serde` — approved. → D51–D53
    - **Final check requested:** evaluate the 12 remarks; then look at the whole engine more broadly (simplifications, responsibilities, dependencies, outdated abstractions, actor/system interaction, ownership/threads/sync, consistency), even radical ideas, but clearly separate "include now" from "later"; no invented work. Then the final spec update; implementation only on a separate command.
11. **2026-11-12. Claude (round 5).**
    - All 12 remarks checked against the code; none creates a problem. Supporting facts: server time is only used to timestamp outgoing messages (so `SessionController` can stamp them); `RemoteMaster` is `unimplemented!`; `world.level` is used after creation only for values that can be copied (stickiness radius, static data, pipelines).
    - Re-evaluated `pending_root_world`: sync loading blocks winit's event pumping, and Windows/macOS then mark the window "Not Responding" (first launch compiles shaders for many seconds), so recommends keeping the async field (§21.10).
    - Made `GpuContext` explicit, inside `RenderScheduler` (§21.7).
    - Rewrote the Rust draft as revision 2 (§21, §22).
    - Broader review (§23): no radical change recommended; six small cleanups proposed for this refactoring (macro-generated `ActorWrapper` impl, remove the self-addressed `SpawnControlledActor`, `InputSystem` without `Arc<Mutex>`, FIFO messages, `EffectsSystem` → function, sync `Engine::new`), plus a list for later and implementation risks.

12. **2026-11-12. Author (round 6): final answers, start authorized.**
    - Cleanups (§23.2): **macro for `ActorWrapper` — yes**; **keep `SpawnControlledActor`** ("I had reasons for doing it that way that I don't remember; maybe a separate pass later"); **remove `Arc<Mutex>` from input — yes, if no complications**; **keep `Vec::pop()`** ("I write actor logic as if all events in a tick happen simultaneously, so order shouldn't matter; `pop()` is fast, no shifting"); **`EffectsSystem` → function — yes**; **`Engine::new` stays async** if wgpu requires it ("don't make synchronicity a goal").
    - **`pending_root_world`: keep** (convinced by the "Not Responding" argument).
    - **`GpuContext`: separate field in `EngineSharedSystems`** ("GPU infrastructure for creating resources vs the system that executes rendering — two responsibilities"; the "can't exist without each other" argument isn't enough to merge them). Handle headless correctly.
    - **File names** `user_settings.json` / `connection_settings.json` approved.
    - **Final spec requested:** re-check the whole history (early ideas and all later changes) and the issue so only final decisions are used; keep the conceptual part (why shared vs world systems, owner-driven simulation, engine not knowing the hierarchy, shared scheduler but per-world render data, avoiding locks/abstractions, session logic in `SessionController`, descriptor vs world, `Messenger`, deferred tasks, …); keep the discussion history; separate **Concept/Rationale**, **Discussion/Decision history**, **Technical spec/Implementation plan**. "We don't save the issue's size."
    - **`AGENTS.md`:** record the author's development principles (simple, no extra abstractions, clear responsibilities, avoid unnecessary locks esp. on hot paths, minimal input/render latency, keep established names, discuss before large changes, don't mix tasks) — only what follows from the conversation; and an instruction to consult relevant issues and their history before proposing changes ("issues are a knowledge base, not just a task list").
    - **Then implement right away**, no further confirmation; free to edit/create/delete files incl. WIP files. Expected result: the game works as now (CTF map, main mechanics) on the new architecture.
13. **2026-11-12. Claude (round 6).**
    - Re-read the issue and the code for the remaining areas (renderer construction, UI renderer, net state machine, matchmaking request, level loading).
    - Found that `UIRenderer` binds world buffers for the HUD scanner → re-binding on present (D56).
    - Found that only the CTF map contains a `SessionController` → network only there after V1 (D57, accepted behaviour change in §24.2).
    - Restructured this issue into Parts I–IV; wrote Part I (concept), revision 3 of the architecture (§21), the spec (§24).
    - Updated `AGENTS.md` with the author's principles and the "consult issues" rule. Started the implementation (§24.9).

---

## 20. Round-4 review: problems in the code the design must handle

Found by re-reading the live code against the agreed design. None of them changes the architecture; each needs a small, explicit decision, already reflected in §21.

### 20.1 `Level` owns constructed actors → the descriptor needs actor *descriptions* [proposed]
- `Level { main_actor: Option<ActorWrapper>, actors: Option<Vec<ActorWrapper>>, .. }`; `World::set_new_level` **takes** them.
- An `Arc<WorldDescriptor>` used for several `World::new` calls can't give out the same actors twice, and `ActorWrapper` isn't `Clone`.
- [proposed] The descriptor stores the map's actor JSON entries (`serde_json::Value`); ~~`World::new` calls the existing `parse_json_main_actor` / `parse_json_actors` on them.~~
- done in round 5 (author): **the creation logic stays encapsulated in `WorldDescriptor`.** `World::new` calls a descriptor method (e.g. `create_actors()`), which builds **new** actor instances from the stored JSON with the existing `parse_json_*` functions and returns them. `World` doesn't know how actors are made from JSON. Construction still happens at `World::new` time (it's called from there), so creation side effects stay in `World::new`.

### 20.2 Actor ids are rewritten to network peer ids [proposed]
- On connection, `NetCommand::NetSystemIsConnectedAndGetNewPeerID(new_id)` → `world.change_actor_id(main_actor_id, new_id)`. Remote messages are addressed by these ids.
- ~~With session logic in `SessionController` (V1), this becomes a world command `CommandType::ChangeActorId { old, new }`.~~ [rejected] round 5.
- done in round 5: `SessionController` → message `ChangeYourId(new_id)` to the main player (id from `WorldContext::main_actor_id`) → the player sends `CommandType::ChangeMyId(new_id)`; old id = `command.sender`. The `Messenger` updates `Actors::main_actor_id` if the changed actor was the main one, and keeps today's collision handling (if `new_id` is taken, the other actor gets a fresh random id).
- Peer connect/disconnect → `SpawnActor(PlayerDoll)` / `RemoveActor`. Server time → kept in `SessionController` (round 5).

### 20.3 Server time lives in `TimeSystem`, actors read it
- 9 actor call sites read `time_system.get_server_time()`, all to timestamp outgoing messages. done in round 5: server time lives in `SessionController`, which stamps those messages itself. See §5.3.

### 20.4 Input routing and nested worlds
- `InputSystem::set_input_to_controlled_actors(&mut World, &mut NetSystem)` writes the frame's input into every `ControlledActor` whose `InputMaster` is local. (`net` is passed but unused.)
- ~~[proposed] `World::simulate` calls it for **every** world. Consequence: an actor with a local `InputMaster` in a nested world will react to the keyboard.~~
- done in round 5: `InputMaster` and `set_input_to_controlled_actors` are removed; actors read `shared.input` directly (Godot-style). See §8.

### 20.5 Window-level hotkeys live in `main_loop.rs`
- `J/K` sensitivity, `N/M` volume, `Y/H` open browser, `Esc/Enter` fullscreen and cursor grab are handled in `Slice4DShooter::window_event`, outside the world.
- [proposed] They stay at the engine/application level and operate on `shared` (`global_settings`, `audio`, `ui`, window). Changes to persistent settings call `GlobalSettings::save()` (§14.3). `i`/`O` (scale) and `P` (shadows) currently go through `RenderSystem::process_player_input`; they move to the same engine-level handler: `i/O` → `root_world.set_output_scale(..)` + save, `P` → `global_settings.shadows_enabled` + save.

### 20.6 Pipelines target the surface format
- `raymarch_target_texture` and the main/minimap pipelines use `config.format` (the surface format, e.g. `Bgra8UnormSrgb`). A nested world has no surface.
- [proposed] All world output textures use **one fixed format** (constant `WORLD_OUTPUT_FORMAT`); only the upscale pipeline targets the surface format. Note: Implementation note: pick the format so that colours stay **identical** (sRGB vs linear). The current path is "raymarch writes into a surface-format texture → upscale samples it → surface"; keeping an `*Srgb` format for the output texture preserves that.

### 20.7 The xwz minimap is world-specific
- The minimap pass uses the level's minimap pipeline and draws into `raymarch_target_texture`. → It's part of the world's `encode` (into its output texture), not of the scheduler.

### 20.8 UI is shared and drawn on top of the presented world
- `UISystem` is shared; `ui.write_buffers_ui(queue, aspect)` is called each frame; `UIRenderer` lives inside `Renderer`.
- [proposed] `UIRenderer` belongs to the render loop and is drawn after the upscale of the **presented** world. `write_buffers_ui` moves to `EngineSharedSystems::post_simulate`.
- Note: UI elements shown by actors of a replaced root world stay visible unless someone hides them. That's today's behaviour as well (level change), and the UI rewrite (out of scope) will fix it with actor-owned UI elements.

### 20.9 `HeadlessEngine` has no users
- No live entry point constructs `HeadlessEngine` (the alternative `*_main.rs` bins are commented out). Headless becomes `Engine::new(None /* no window */, ..)` with `render: None`. [proposed] Verify it at least once (e.g. a tiny headless run) since nothing exercises it today.

### 20.10 Level loading commands from actors (done in round 5)
- `FinalTrigger` uses `PreloadNewLevelAsync` (at start), then `LoadNewLevelAsync` or `LoadNewLevelSync` after the level is beaten.
- [proposed] New flow, same behaviour, no engine commands:
  - at start: `self.next = Some(shared.runtime.spawn_blocking(move || WorldDescriptor::load(&name, kit)))` (preload)
  - when it's time: if the handle is finished, take the descriptor and send `OuterWorldMessage::ReplaceRootWorld(Arc::new(desc))`; otherwise keep waiting (async) or `block_on` it (sync variant)
  - `CommandType::{LoadNewLevelSync, LoadNewLevelAsync, PreloadNewLevelAsync}` and `World.preloaded_levels` are deleted.
- done in round 5 (author): `FinalTrigger` is a test actor that won't be widely used, but the replacement is right. **Target flow once the Resource System exists:**
  1. On creation, `FinalTrigger` requests the next world's descriptor from the Resource System.
  2. If it was loaded before, the Resource System returns it; otherwise it starts an async load and the actor gets a handle.
  3. When it's time to change levels, the actor sends `OuterWorldMessage::ReplaceRootWorld(..)` carrying the prepared descriptor (or a reference to it: `Arc`/`Box`, so the enum stays small).
  4. No engine-level `PreloadNewLevelAsync` / `LoadNewLevel*` concepts are needed; they become redundant.
- For now: prepare the architecture for this and implement the minimum that keeps today's behaviour (`WorldDescriptor::load` on `spawn_blocking` + `JoinHandle`, as above).

### 20.11 Smaller items
- `CommandType::SpawnEffect` is unused → delete. `EffectsSystem` → free function (`effects::spawn_wave(messenger, ..)`).
- `Engine::new` is `async` only because `Level::load_level` / `PlayerSettings::load` / `RenderSystem::new` are; it's run with `pollster::block_on`. Fine to keep; not a goal of the refactoring.
- `NetSystem` currently receives `ActionsFrameState` (for "press P to play online") and `UISystem`; both move into `SessionController`, which has `shared.input` and `shared.ui`.

---

## 23. Round-5 broader engine review [proposed]

The author asked for a last, wider look at the whole engine: not only "how to implement the agreed refactoring", but whether there are fundamental flaws worth fixing now, while everything is being touched anyway, without inventing extra work.

### 23.1 Overall verdict

**After the agreed refactoring the architecture is good enough for further development. I don't see a reason for a radical change.** In particular:
- The **actor + message** model is a good fit for this game (few dozen actors, lots of message-driven gameplay, network messages mapping onto actor messages). An ECS would be a rewrite of every actor for no concrete gain here. [rejected] Not recommended.
- **Static dispatch via `ActorWrapper`** stays (it's also used for pattern matches like `RemoveAllHolesAndEffects`). `Box<dyn Actor>` would lose those matches and gain nothing important.
- The biggest structural problems (systems passed as 7–11 separate parameters, session logic in the transport, gameplay constants in a settings file, one hard-coded world, the render loop on a tokio worker) are exactly what this refactoring fixes.

### 23.2 Recommended to include in this refactoring (small, and the code is touched anyway)

done in round-6 outcome (author): items 1, 3, 5 **included**; item 2 (`SpawnControlledActor`) and item 4 (FIFO) **not included**; item 6 (sync `Engine::new`) **not included** (wgpu adapter/device creation is async; `async fn new` stays). See D54.

1. **Generate the `impl Actor for ActorWrapper` matches with a `macro_rules!`.**
   - Today: ~1300 lines in `actor.rs`, ~420 `ActorWrapper::` arms over ~12 methods. Adding an actor means editing every match (see `AGENTS.md`).
   - The refactoring changes **every** method signature, so every arm gets rewritten anyway. A small declarative macro (`actor_wrapper! { MainPlayer(MainPlayer), Flag(Flag), … }`) that generates the enum and the forwarding impl removes that boilerplate in the same pass. Adding an actor then means one line.
   - Not a new abstraction: the same enum, the same static dispatch, only generated. No new dependency (alternative: the `enum_dispatch` crate, but a 30-line local macro is enough).
2. **Remove `SpawnControlledActor`.** All 5 uses are `MainPlayer` sending this command **to itself** (`sender == target == self.id`), so that the engine calls `ControlledActor::spawn(physics, ui, audio, engine_handle)` on it. It was a workaround for the actor not having the systems at hand. Now it has `world` + `shared`, so it calls its own `spawn(..)` directly. `ControlledActor` shrinks to what the renderer needs (camera, screen effects, team).
3. **`InputSystem` without `Arc<Mutex<…>>`.** The actions table is `HashMap<SomeButton, Arc<Mutex<(ButtonActions, Action, i32)>>>`, but input is only used on the main thread. Plain values; compute `ActionsFrameState` once per tick in `pre_simulate` (`shared.input.actions()`) instead of on every `get_input()` call. Fits the round-5 input decision.
4. **FIFO message order.** The pump uses `Vec::pop()`, so messages in a buffer are delivered **newest first**. Since the pump moves into `Messenger` anyway, deliver in send order. Risk is low (gameplay doesn't rely on reverse order as far as the code shows), but it's a behaviour change, so it'll be tested specifically. If something depends on LIFO, keep LIFO and note it.
5. **`EffectsSystem` → free function** (already agreed): an empty struct passed to every actor.
6. **`Engine::new` without `async`.** It's `async` only because of `Level::load_level`, `PlayerSettings::load` and `RenderSystem::new`, and it's run with `pollster::block_on`. In the new structure only wgpu adapter/device creation is async; `pollster::block_on` there and a plain `fn new`. Cosmetic, but it removes `async` from the whole construction chain.

### 23.3 Noted for later (not in this refactoring)

- **`Option<ActorID>` + `.expect("… have not ActorID")` everywhere.** Actors get their id only when added to the world. Assigning the id at construction would remove hundreds of `expect`s, but touches every actor's logic. Later, separately.
- **Panics as error handling** (`unwrap`/`expect` on I/O, network, parsing). Typical for a prototype; for a release-quality engine, errors in loading/networking should become `Result`s with logged fallbacks. Gradually.
- **UI**: global `UIElementType` HUD elements in a shared map; actors toggle them. Already out of scope (Fyrox UI rewrite).
- **Spatial audio in nested worlds** uses the nested world's coordinates. The owner/actor is responsible for now; a per-world "audio listener" may be needed with in-world screens.
- **Main loop pacing**: `ControlFlow::WaitUntil(now + target)` is set at the start of the tick, and the Windows timer-resolution call (`timeBeginPeriod`) is commented out. Not related to this refactoring; can be looked at when simulation/render rates become configurable.
- **`recieve_message` / other misspellings** (`Envirnoment`, `Trgger`, `Postitions`): leave; renaming would add noise to an already huge diff.
- **Torn frames between buffer writes** (§9.4.1 item 4): as today.

### 23.4 Risks to watch during implementation

- **Colours**: keep sRGB handling identical when switching to the fixed output format (§20.6). Compare screenshots before/after.
- **Message order** (§23.2.4).
- **Network**: `SessionController` taking over the ~950-line `process_message` is the riskiest part; it needs a real two-client test against a local `game_server`, since there are no automated tests.
- **Headless** isn't exercised by any entry point (§20.9).
- **Diff size**: every actor changes. The staging (§15) keeps the game playable after each stage, so each stage is checked by building and playing.

---

# Part III — Final target architecture

## 21. Final target architecture (Rust), revision 3 (done in round 6)

Revision 1 (round 4) → reviewed → revision 2 (round 5) → reviewed → **revision 3 (round 6, final)**. Names, fields and signatures are what the implementation follows; method bodies are only shown where they explain an interaction. Module paths are indicative. Small deviations found necessary during implementation are recorded in §24.9 (implementation log).

### 21.0a Changes since revision 2 (round 6)

| # | Revision 2 | Revision 3 | Who |
|---|---|---|---|
| 1 | `GpuContext` inside `RenderScheduler` | **Separate field `shared.gpu: Option<GpuContext>`**; `RenderScheduler` holds a clone | author |
| 2 | `SpawnControlledActor` removed, `ControlledActor::spawn` removed | **Kept as today** (new signature only); removal maybe in a later pass | author |
| 3 | FIFO message order | **Keep `Vec::pop()` (LIFO)** | author |
| 4 | `Engine::new` synchronous | **Stays `async`** (wgpu adapter/device creation is async); `pollster::block_on` at the call site as today | author |
| 5 | — | `UIRenderer` re-binds the **presented world's** buffers (the HUD scanner reads world buffers) | Claude (found in code) |
| 6 | settings file names proposed | `user_settings.json`, `connection_settings.json` [accepted] | author |
| 7 | — | `NetSystem` transport details: WebSocket and WebRTC connections behind one handle API; the matchmaking request moves into `SessionController` | Claude (detail) |
| 8 | — | Networking exists only in worlds with a `SessionController` (today: the CTF map) | consequence of V1, see §24.7 |


### 21.0 Changes since revision 1 (round 5)

| # | Revision 1 | Revision 2 | Who |
|---|---|---|---|
| 1 | `World::new` parses actor JSON | `WorldDescriptor::create_actors()` builds actors; `World` doesn't know about JSON | author |
| 2 | `CommandType::ChangeActorId { old, new }` | `ChangeYourId(new)` message → `CommandType::ChangeMyId(new)`; old id = `command.sender` | author |
| 3 | Message pump in `World::process_messages` | **All message delivery and command execution inside `Messenger`**; `World` only calls it | author |
| 4 | Server time in `GlobalTimeSystem` | Server/session time inside `SessionController`, which stamps outgoing messages | author |
| 5 | `set_input_to_controlled_actors` per world | `InputMaster` removed; actors read `shared.input` (Godot-style) | author |
| 6 | `World.descriptor: Arc<WorldDescriptor>`, `WorldContext.descriptor` | Removed. The descriptor is only used in `World::new`; the few values needed later are copied into the systems that use them | author |
| 7 | Device/queue hidden inside `RenderScheduler` | Explicit **`GpuContext`** (Clone), owned by `RenderScheduler`, exposed via `render.gpu()` | author's question |
| 8 | Static splash | Splash is a small render-loop component that gets the time each frame, so an animated indicator can be added later | author |
| 9 | `on_level_loaded`, `send_boardcast_message` | `on_world_started`, `send_broadcast_message` | author |
| 10 | Settings: open questions | User file (platform config dir) for user preferences only; connection data compiled in, overridable only next to the binary; `serde` derive | author |
| 11 | `SpawnControlledActor` command; `ControlledActor::{get_input_master, spawn}` | Command removed: the player calls its own `spawn(..)` directly; `ControlledActor` keeps only what the renderer needs | Claude (§23) — [rejected] reverted in revision 3: kept |
| 12 | `pending_root_world` | Kept, after re-evaluation (§21.10) | Claude, at the author's request |

### 21.1 `Engine` (`src/engine.rs`)

```rust
pub struct Engine
{
    // Field order = drop order (Rust drops fields top to bottom):
    // worlds first, then shared systems (RenderScheduler joins its thread in Drop),
    // the window last, after the render thread has released its Arc<Window>.
    root_world: Option<World>,                                  // None while the initial world loads → splash
    pending_root_world: Option<tokio::task::JoinHandle<WorldDescriptor>>,   // §21.10
    pub shared: EngineSharedSystems,
    window: Option<Arc<Window>>,                                // None = headless
    quit_requested: bool,
}

pub struct EngineConfig
{
    pub initial_world: String,          // map name; now "capture_the_flag_map", later the main menu
    pub specific_backend: Option<wgpu::Backend>,
    pub with_ui_renderer: bool,
    pub disable_net_system: bool,
    pub game_mode: GameMode,
}

impl Engine
{
    pub async fn new(window: Option<Window>, config: EngineConfig) -> Engine;   // async because of wgpu
    pub fn tick(&mut self);                                     // one engine tick
    pub fn on_window_resized(&mut self, size: PhysicalSize<u32>);
    pub fn quit_requested(&self) -> bool;

    fn poll_pending_root_world(&mut self);
    fn set_root_world(&mut self, descriptor: Arc<WorldDescriptor>);
    fn handle_outer_world_message(&mut self, message: OuterWorldMessage);
}
```

`Slice4DShooter` (winit `ApplicationHandler`, `main_loop.rs`) owns `Option<Engine>`, forwards window/device events to `engine.shared.input` and `engine.on_window_resized`, handles the window-level hotkeys (§20.5), calls `engine.tick()` and exits when `quit_requested()`. `HeadlessEngine` is deleted.

### 21.2 `EngineSharedSystems` (`src/engine/shared_systems.rs`)

```rust
pub struct EngineSharedSystems
{
    pub global_settings: GlobalSettings,
    pub time: GlobalTimeSystem,
    pub input: InputSystem,
    pub net: NetSystem,
    pub audio: AudioSystem,
    pub ui: UISystem,
    pub gpu: Option<GpuContext>,            // None = headless (§21.7); revision 3: separate field
    pub render: Option<RenderScheduler>,    // None = headless; owns the render thread
    pub runtime: tokio::runtime::Runtime,   // net tasks, async loading; dropped last
}

impl EngineSharedSystems
{
    pub async fn new(window: Option<Arc<Window>>, config: &EngineConfig) -> Self;   // async: wgpu adapter/device

    /// Before the root world's simulate.
    pub fn pre_simulate(&mut self)
    {
        self.input.collect_gamepad_button_input();
        self.time.start_of_frame();
        self.net.tick();                    // poll sockets, connection states, buffer received packets
    }

    /// After the root world's simulate.
    pub fn post_simulate(&mut self)
    {
        self.net.flush();                   // no-op while sends are immediate (§13.5)
        if let (Some(gpu), Some(render)) = (&self.gpu, &self.render)
        {
            self.ui.write_buffers_ui(&gpu.queue, render.surface_aspect());
        }
        self.input.reset_input();
        self.time.end_of_frame();
    }
}
```

Access is plain `pub` fields (`shared.audio.…`, `shared.input.actions()`), no getters or traits.

### 21.3 `World`, `Actors`, `WorldContext`, `Actor` (`src/engine/world.rs`, `world/actors.rs`)

```rust
pub struct World
{
    actors: Actors,
    messenger: Messenger,
    physics: PhysicsSystem,
    time: WorldTimeSystem,
    render: Option<WorldRenderSystem>,  // None: headless engine, or rendering disabled for this world
    // no descriptor: it's only needed in World::new (round 5)
}

pub struct WorldConfig
{
    pub render: Option<WorldRenderConfig>,  // None = this world is never rendered
}

pub struct WorldRenderConfig
{
    pub resolution: UVec2,              // output texture resolution (any aspect ratio)
    pub scale: f32,                     // output texture scale factor
    pub render_rate: u32,               // 0 paused, 1 every render frame, N every Nth
}

impl World
{
    pub fn new(descriptor: &WorldDescriptor, config: WorldConfig, shared: &mut EngineSharedSystems) -> World;
    pub fn simulate(&mut self, shared: &mut EngineSharedSystems);

    // rendering control (&self: they only store atomics)
    pub fn output_texture_id(&self) -> Option<WorldTextureId>;  // None if not rendered
    pub fn set_render_rate(&self, rate: u32);
    pub fn set_output_resolution(&self, resolution: UVec2);
    pub fn set_output_scale(&self, scale: f32);
    pub fn set_output_resolution_and_scale(&self, resolution: UVec2, scale: f32);

    // owner → world (sender id = OUTER_WORLD_SENDER_ID = 0)
    pub fn send_broadcast_message(&mut self, message: MessageType);
    pub fn send_direct_message(&mut self, to: ActorID, message: MessageType);

    // world → owner
    pub fn take_outer_world_messages(&mut self) -> Vec<OuterWorldMessage>;
}

/// Former `World` (the actor container).
pub struct Actors
{
    actors: HashMap<ActorID, ActorWrapper>,
    main_actor_id: ActorID,             // the camera / HUD actor (today's World.main_actor_id)
}

impl Actors
{
    pub fn insert(&mut self, actor: ActorWrapper) -> ActorID;   // assigns a random non-zero id if needed
    pub fn remove(&mut self, id: ActorID) -> Option<ActorWrapper>;
    pub fn change_id(&mut self, old: ActorID, new: ActorID);    // keeps today's collision handling
    pub fn main_actor_id(&self) -> ActorID;
    pub fn get(&self, id: ActorID) -> Option<&ActorWrapper>;
    pub fn get_mut(&mut self, id: ActorID) -> Option<&mut ActorWrapper>;
    pub fn iter(&self) -> impl Iterator<Item = (&ActorID, &ActorWrapper)>;
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&ActorID, &mut ActorWrapper)>;
}

/// The per-world part passed to actors. Built from the World's fields (split borrow).
pub struct WorldContext<'a>
{
    pub messenger: &'a mut Messenger,
    pub physics: &'a PhysicsSystem,
    pub time: &'a WorldTimeSystem,
    pub main_actor_id: ActorID,         // read-only; e.g. SessionController → ChangeYourId
}

pub trait Actor
{
    fn tick(&mut self, world: &mut WorldContext, shared: &mut EngineSharedSystems) {}
    fn recieve_message(&mut self, message: Message, world: &mut WorldContext, shared: &mut EngineSharedSystems) {}
    fn on_added_to_world(&mut self, world: &mut WorldContext, shared: &mut EngineSharedSystems) {}
    fn on_world_started(&mut self, world: &mut WorldContext, shared: &mut EngineSharedSystems) {}  // was on_level_loaded
    fn get_transform(&self) -> &Transform;
    fn get_mut_transform(&mut self) -> &mut Transform;
    fn get_physical_element(&mut self) -> Option<PhysicalElement> { None }
    fn get_visual_element(&self) -> Option<VisualElement> { None }
    fn get_id(&self) -> Option<ActorID>;
    fn set_id(&mut self, id: ActorID);  // called by Actors/Messenger only
    fn get_actor_as_controlled(&self) -> Option<&dyn ControlledActor> { None }
    fn get_actor_as_controlled_mut(&mut self) -> Option<&mut dyn ControlledActor> { None }
}
// delta: world.time.delta(); input: shared.input.actions()

/// The main (camera) actor: what the renderer needs + spawn (kept, revision 3).
/// get_input_master is removed (InputMaster is gone).
pub trait ControlledActor
{
    fn get_camera(&self) -> Camera;
    fn get_screen_effects(&self) -> &PlayerScreenEffects;
    fn get_team(&self) -> Team;
    fn spawn(&mut self, world: &mut WorldContext, shared: &mut EngineSharedSystems);   // used by SpawnControlledActor
}
```

The `recieve_message` misspelling stays (used everywhere; out of scope). `boardcast` is fixed because the `Messenger` API is renamed anyway.

### 21.4 `Messenger` (`src/engine/world/messenger.rs`, former `EngineHandle` + the message pump)

done in round 5 (author): **all logic of delivering and processing messages between actors is encapsulated in `Messenger`.** It gets access to `Actors` and the other world systems it needs; `World` only calls its methods. Today this logic is spread over `engine.rs` (`send_messages_and_process_commands`, `send_boardcast_messages`, `send_direct_messages`, `execute_command`) and `World` (`add_actor_to_world`, …).

```rust
pub const OUTER_WORLD_SENDER_ID: ActorID = 0;   // actor ids are random non-zero (get_random_non_zero_id)

pub struct Messenger
{
    broadcast_messages: Vec<Message>,
    direct_messages: Vec<(ActorID, Message)>,
    commands: Vec<Command>,
    outer_world_messages: Vec<OuterWorldMessage>,
}

impl Messenger
{
    // used by actors (through WorldContext) and by the World's owner
    pub fn send_broadcast_message(&mut self, message: Message);
    pub fn send_direct_message(&mut self, to: ActorID, message: Message);
    pub fn send_command(&mut self, command: Command);
    pub fn send_to_outer_world(&mut self, message: OuterWorldMessage);
    pub fn take_outer_world_messages(&mut self) -> Vec<OuterWorldMessage>;

    /// The message pump: deliver broadcasts, direct messages and execute commands until all buffers are empty.
    pub fn process_messages(
        &mut self,
        actors: &mut Actors,
        physics: &PhysicsSystem,
        time: &WorldTimeSystem,
        shared: &mut EngineSharedSystems,
    );

    /// Adds actors (used by World::new and by the SpawnActor command): insert, then on_added_to_world.
    pub fn add_actors(&mut self, new_actors: Vec<ActorWrapper>, actors: &mut Actors,
                      physics: &PhysicsSystem, time: &WorldTimeSystem, shared: &mut EngineSharedSystems);

    /// Like add_actors, and marks it as Actors::main_actor_id.
    pub fn add_main_actor(&mut self, actor: ActorWrapper, actors: &mut Actors,
                          physics: &PhysicsSystem, time: &WorldTimeSystem, shared: &mut EngineSharedSystems);

    /// Calls on_world_started on every actor (end of World::new).
    pub fn start_world(&mut self, actors: &mut Actors, physics: &PhysicsSystem,
                       time: &WorldTimeSystem, shared: &mut EngineSharedSystems);

    fn execute_command(&mut self, command: Command, actors: &mut Actors, physics: &PhysicsSystem,
                       time: &WorldTimeSystem, shared: &mut EngineSharedSystems);
}

pub struct Command { pub sender: ActorID, pub command_type: CommandType }

pub enum CommandType
{
    SpawnActor(ActorWrapper),
    SpawnControlledActor(ActorID),      // kept (revision 3)
    RemoveActor(ActorID),
    ChangeMyId(ActorID),                // round 5: an actor changes only its own id; old = command.sender
    RemoveAllHolesAndEffects,
    // removed:
    //   NetCommand(..)                          → SessionController (V1)
    //   LoadNewLevelSync/Async, PreloadNewLevelAsync → WorldDescriptor + OuterWorldMessage::ReplaceRootWorld
    //   ShowConnectionStatusUI                  → message to SessionController
    //   SpawnEffect                             → unused
}
```

How `process_messages` hands `&mut Messenger` to actors while it is itself running: each message is popped out of the buffer first (owned value), then delivered with `WorldContext { messenger: self, .. }`. Actors push new messages into the same buffers; the loop continues until all are empty. No extra structures needed.

Message order: **`Vec::pop()` (LIFO) as today** [accepted] (revision 3, author): actor logic treats everything within one tick as simultaneous; `pop()` is fast and simple.

### 21.5 `OuterWorldMessage` [accepted] (accepted in round 5)

```rust
/// Requests from a world to whoever owns it: the Engine for the root world,
/// the owner actor for a nested world. The receiver decides what to do.
pub enum OuterWorldMessage
{
    /// Close the application. Root: the engine exits. Nested: the owner may forward it outward.
    QuitApplication,
    /// Replace the root world with a prepared one. Root: the engine does it.
    /// Nested: the owner may forward it to its own world's outer mailbox.
    ReplaceRootWorld(Arc<WorldDescriptor>),
    /// This world has finished its job; the owner may drop it. Root: ignored.
    WorldFinished,
    /// Game-specific messages for owner actors. The engine ignores them (logs).
    Specific(SpecificOuterWorldMessage),
}

/// Hand-written, like SpecificActorMessage; extended as mechanics need it.
pub enum SpecificOuterWorldMessage
{
    TutorialDemoFinished,               // example
}
```

It mirrors the existing `MessageType::{CommonActorsMessages, SpecificActorMessage}` split, so it adds no new concept. The set of messages grows as needed.

### 21.6 `WorldDescriptor` (`src/engine/world/world_descriptor.rs`, former `Level`)

```rust
/// Immutable, shareable (Arc) data prepared in advance. Many Worlds can be created from one descriptor.
pub struct WorldDescriptor
{
    pub name: String,

    // parsed map: geometry and visuals (today's Level fields)
    pub static_objects: Vec<StaticObject>,
    pub all_shapes_stickiness_radius: f32,
    pub visual_materials: Vec<ObjectMaterial>,
    pub blue_players_visual_materials: (i32, i32),
    pub red_players_visual_materials: (i32, i32),
    pub visual_settings_of_environment: EnvirnomentVisualSettings,
    pub red_base_position: Vec4,
    pub blue_base_position: Vec4,
    pub is_generated_raymarch_shader: bool,
    pub it_is_2d_3d_example: bool,

    // actor descriptions (map JSON entries); private: only create_actors() knows how to use them
    main_actor_description: serde_json::Value,
    actor_descriptions: Vec<serde_json::Value>,

    // expensive GPU part; None in headless mode
    pub gpu: Option<WorldDescriptorGpu>,
}

pub struct WorldDescriptorGpu
{
    pub main_pipeline: wgpu::RenderPipeline,
    pub minimap_pipeline: Option<wgpu::RenderPipeline>,
}

impl WorldDescriptor
{
    /// Blocking: read the map (same path fallbacks + embedded maps as today), parse it,
    /// generate/compile shaders and create pipelines if `gpu` is Some.
    /// For async loading run it via `shared.runtime.spawn_blocking`.
    pub fn load(name: &str, gpu: Option<GpuContext>) -> WorldDescriptor;

    /// Round 5 (author): builds NEW actor instances from the stored descriptions with the
    /// existing parse_json_* functions. Called by World::new. Returns (main actor, other actors).
    pub fn create_actors(&self) -> (ActorWrapper, Vec<ActorWrapper>);
}
```

Values needed after creation are **copied** into the systems that use them (that's why `World` doesn't keep the descriptor):
- `PhysicsSystem::new(&descriptor)`: static colliders, `all_shapes_stickiness_radius` (today copied in `update_level_static_info`)
- `WorldRenderSystem::new(&descriptor, ..)`: static render data, stickiness radius (today read from `world.level` every frame in `dynamic_render_data.rs`), `is_generated_raymarch_shader`, pipelines (cheap `Clone` handles)
- actors: `create_actors()` passes what they need at construction, e.g. `it_is_2d_3d_example` to `SessionController`, base positions to flags (already the case)

### 21.7 Rendering (`src/engine/render/`)

#### GPU context [accepted] (revision 3: separate shared field)

```rust
/// The shared wgpu objects needed to create GPU resources.
/// Clone + Send (wgpu 27 handles are cheap clones), so it can go to a loading thread.
#[derive(Clone)]
pub struct GpuContext
{
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub output_format: wgpu::TextureFormat,       // WORLD_OUTPUT_FORMAT, fixed for all worlds (§20.6)
    pub raymarch_bind_group_layouts: [wgpu::BindGroupLayout; 2],
    pub raymarch_pipeline_layout: wgpu::PipelineLayout,
}

impl GpuContext
{
    /// Creates instance, surface, adapter (compatible with the surface), device/queue (same fallbacks as today),
    /// the shared layouts. Returns the surface separately: it belongs to the RenderScheduler.
    pub async fn new(window: &Arc<Window>, backend: Option<wgpu::Backend>) -> (GpuContext, wgpu::Surface<'static>);
}
```

- **Where:** `EngineSharedSystems.gpu: Option<GpuContext>` (author's decision: GPU infrastructure and render execution are different responsibilities). `RenderScheduler` keeps its own clone.
- **Headless:** no window → `gpu: None` and `render: None` (both). Code that needs the GPU matches `shared.gpu`.
- **Who uses it:**
  - `WorldDescriptor::load(name, shared.gpu.clone())` builds pipelines (on a loading thread)
  - `WorldRenderSystem::new(descriptor, config, gpu, scheduler)` creates the world's buffers, bind groups and output texture, then registers them with the scheduler
  - `UISystem::write_buffers_ui` uses `gpu.queue`
- It replaces today's `MainShaderRenderPipelineBuilderKit` (device + layouts clone) and `RenderSystem.renderer_queue`.

#### Shared, main-thread side

```rust
pub struct RenderScheduler
{
    gpu: GpuContext,                    // clone of shared.gpu
    to_render_loop: std::sync::mpsc::Sender<RenderLoopMessage>,
    render_thread: Option<std::thread::JoinHandle<()>>,
    next_texture_id: u64,
    surface_size: PhysicalSize<u32>,
}

impl RenderScheduler
{
    pub fn new(gpu: &GpuContext, surface: wgpu::Surface<'static>, window: Arc<Window>,
               config: &EngineConfig, ui: &UISystem) -> Self;   // spawns the thread; splash shows at once
    pub fn surface_aspect(&self) -> f32;
    pub fn register_world(&mut self, gpu: WorldGpu, shared: &Arc<WorldRenderShared>) -> WorldTextureId;
    pub fn present_world(&self, id: Option<WorldTextureId>);    // None → splash
    pub fn resize_surface(&mut self, size: PhysicalSize<u32>);
}

impl Drop for RenderScheduler
{
    fn drop(&mut self)
    {
        let _ = self.to_render_loop.send(RenderLoopMessage::Shutdown);
        if let Some(thread) = self.render_thread.take()
        {
            if thread.join().is_err() { log::error!("render thread panicked"); }
        }
    }
}

enum RenderLoopMessage
{
    RegisterWorld(WorldSlot),
    PresentWorld(Option<WorldTextureId>),
    ResizeSurface(PhysicalSize<u32>),
    Shutdown,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct WorldTextureId(u64);
```

#### Shared between a world and its slot (atomics only)

```rust
pub struct WorldRenderShared
{
    render_rate: AtomicU32,   // 0 paused, 1 every render frame, N every Nth (Relaxed)
    output: AtomicU64,        // packed width u16 | height u16 | scale f32 bits (Relaxed, one store)
}
```

#### Per world, main-thread side (owned by `World`)

```rust
pub struct WorldRenderSystem
{
    shared: Arc<WorldRenderShared>,  // World dropped → this dropped → slot's Weak dies → slot removed
    texture_id: WorldTextureId,
    queue: wgpu::Queue,
    buffers: WorldGpuBuffers,        // clones of the slot's buffers, written via queue.write_buffer
    render_data: RenderData,         // static + dynamic data computed from actors (as today)
    stickiness_radius: f32,          // copied from the descriptor
    is_generated_raymarch_shader: bool,
}

impl WorldRenderSystem
{
    pub fn new(descriptor: &WorldDescriptor, config: &WorldRenderConfig, gpu: &GpuContext,
               scheduler: &mut RenderScheduler) -> Self;
    /// End of simulate(): compute dynamic render data from actors and write the buffers (today's send_data_to_renderer).
    pub fn update(&mut self, actors: &Actors, time: &WorldTimeSystem, settings: &GlobalSettings);
}

pub struct WorldGpuBuffers { /* today's RendererBuffers */ }
```

#### Render-thread side (owned exclusively by the render loop, no locks)

```rust
struct RenderLoop
{
    inbox: mpsc::Receiver<RenderLoopMessage>,
    gpu: GpuContext,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    upscale: UpscalePass,                     // presented world's output → surface
    ui_renderer: Option<UIRenderer>,          // re-bound to the presented world's buffers (HUD scanner), see below
    splash: SplashScreen,
    placeholder_texture: wgpu::TextureView,   // 1×1, for unresolved WorldTextureIds
    slots: Vec<WorldSlot>,                    // registration order; rendered in reverse
    presented: Option<WorldTextureId>,
    pacing: FramePacing,                      // target 0.008 s, stats (today's fields in Renderer)
}

/// Round 5: a small component, not a World. Starts as the logo texture drawn through the
/// upscale pipeline; gets the time every frame, so an animated indicator (e.g. a rotating
/// spinner in its own tiny shader) can be added later without touching anything else.
/// It keeps animating while the main thread is busy, because it runs on the render thread.
struct SplashScreen
{
    logo: wgpu::TextureView,
    // later: spinner pipeline + uniform buffer
}

impl SplashScreen
{
    fn encode(&mut self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView, time: f32);
}

struct WorldSlot
{
    id: WorldTextureId,
    shared: Weak<WorldRenderShared>,
    gpu: WorldGpu,
    frame_counter: u32,
}

/// GPU objects of one world. Created on the main thread in WorldRenderSystem::new,
/// moved into the slot, then touched only by the render thread (no Mutex).
struct WorldGpu
{
    main_pipeline: wgpu::RenderPipeline,
    minimap_pipeline: Option<wgpu::RenderPipeline>,
    bind_group_0: wgpu::BindGroup,
    bind_group_1: wgpu::BindGroup,
    sky_box: wgpu::TextureView,
    output: OutputTexture,
}

struct OutputTexture
{
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    size: UVec2,         // resolution × scale; recreated when the packed atomic changes
    generation: u32,     // bumped on recreation; consumers rebuild bind groups
}

impl WorldGpu
{
    fn encode(&mut self, encoder: &mut wgpu::CommandEncoder);   // raymarch (+ minimap) into output
}

impl RenderLoop
{
    fn run(mut self)
    {
        loop
        {
            // drain inbox: Register → slots.push; Present → presented; Resize → reconfigure;
            //              Shutdown or disconnected → return (drops surface, slots, UI renderer)
            // pacing + "no frame queueing" (device.poll → QueueEmpty), as today
            // retain slots whose Weak upgrades; for each due slot (render_rate, frame_counter), in reverse:
            //     resize output if the packed size changed; gpu.encode(&mut encoder)
            // presented slot's output → upscale to surface, or splash.encode(..) if none
            // UI → one submit → present
            // short sleep if the iteration was quick (~3 ms, as today)
        }
    }

    /// For future consumers (in-world screens): resolve a world's texture on the render thread.
    fn texture_view(&self, id: WorldTextureId) -> (&wgpu::TextureView, u32 /*generation*/);  // placeholder if missing
}
```

**UI renderer and the presented world** (revision 3, found in the code): `UIRenderer::new` today receives the world's `other_dynamic_data`, `player_forms` and `dynamic_*_shapes` buffers, because the HUD **scanner** element reads the world's dynamic data. So:
- `WorldGpu` keeps clones of its buffers (`WorldGpuBuffers`).
- On `PresentWorld(id)` the render loop calls `ui_renderer.bind_world_buffers(&slot.gpu.buffers)`, which rebuilds the scanner bind groups. Until a world is presented (splash), scanner elements are skipped.
- Image/progress-bar UI elements don't depend on a world and are created once.

### 21.8 Time [accepted]

```rust
pub struct GlobalTimeSystem
{
    pub target_frame_duration: Duration,  // engine tick, 60 Hz
    app_start: Instant,
    frame_start: Instant,
    prev_frame_duration: f32,
    frame_counter: u64,                   // engine ticks since start
    // no server time (round 5: it belongs to SessionController)
}

impl GlobalTimeSystem
{
    pub fn start_of_frame(&mut self);
    pub fn end_of_frame(&mut self);
    pub fn frame_start(&self) -> Instant;
    pub fn prev_frame_duration(&self) -> f32;   // clamped as today
    pub fn frame_counter(&self) -> u64;
    pub fn time_since_start(&self) -> Duration;
    pub fn now_utc(&self) -> SystemTime;
}

pub struct WorldTimeSystem
{
    last_simulate_frame_start: Option<Instant>,
    delta: f32,
    simulate_counter: u64,
    world_time: f64,                      // simulated seconds since creation
}

impl WorldTimeSystem
{
    pub fn begin_simulate(&mut self, global: &GlobalTimeSystem);  // computes delta (§5.3)
    pub fn delta(&self) -> f32;
    pub fn simulate_counter(&self) -> u64;
    pub fn world_time(&self) -> f64;
}
```

### 21.9 `NetSystem` (transport), `SessionController`, `InputSystem`

```rust
pub struct NetSystem
{
    runtime: tokio::runtime::Handle,      // clone of the shared runtime's handle, for socket tasks
    connections: HashMap<ConnectionHandle, Connection>,
    next_handle: u32,
    disabled: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ConnectionHandle(u32);

pub enum ConnectionConfig
{
    WebSocket { url: String, connect_timeout: Duration },        // matchmaking (tokio-tungstenite)
    WebRtc { url: String, ice: IceServersConfig },               // game server (matchbox; reliable ch 0, unreliable ch 1)
}

pub enum ConnectionState
{
    Connecting,
    Connected,
    Disconnected(DisconnectReason),
}

pub enum DisconnectReason { Timeout, ClosedByRemote, Lost(String), ClosedLocally }
pub enum Reliability { Reliable, Unreliable }
pub type Packet = Box<[u8]>;

impl NetSystem
{
    pub fn connect(&mut self, config: ConnectionConfig) -> ConnectionHandle;
    pub fn state(&self, handle: ConnectionHandle) -> ConnectionState;
    pub fn send(&mut self, handle: ConnectionHandle, packet: Packet, reliability: Reliability);
    pub fn receive(&mut self, handle: ConnectionHandle) -> Vec<Packet>;
    pub fn local_peer_id(&self, handle: ConnectionHandle) -> Option<u128>;   // WebRTC: our id in the matchbox network
    pub fn close(&mut self, handle: ConnectionHandle);
    pub fn tick(&mut self);               // pre_simulate: poll matchbox peers/channels, drain WebSocket task events
    pub fn flush(&mut self);              // post_simulate: no-op (sends are immediate)
}
```

Transport details (revision 3):
- **WebSocket:** `connect` spawns a tokio task that connects (with timeout), then forwards packets both ways over two `tokio::sync::mpsc` unbounded channels. `tick()` drains the task's events (`Connected`, `Packet`, `Closed(reason)`) into the connection's state and receive buffer.
- **WebRTC:** `connect` builds the `WebRtcSocket` exactly as `net.rs` does today (reconnect attempts, keep-alive, ICE/TURN, one reliable + one unreliable channel) and spawns its future. `tick()` calls `try_update_peers()`: the first connected peer is the game server (client-server topology, as today); a closed channel or a disconnected peer → `Disconnected`. Packets from both channels go to one receive buffer. `send` picks the channel by `Reliability` and sends to the server peer.
- What moves out of `NetSystem` into `SessionController`: the connection state machine (`WaitingForUsersRequest → ConnectingToMatchmakingServer → ConnectingToGameServer → ConnectedToGameServer / ConnectionFailure`), attempts and timeouts, the **matchmaking request/response** (`get_game_server_url`: version + game type → server address / no free servers / wrong version), `ServerMessage` decoding and `process_message`, the connection-status UI, the player id list, and session time.

```rust
pub struct SessionController
{
    id: Option<ActorID>,
    state: SessionState,                  // today's ConnectionState machine from net.rs
    matchmaking: Option<ConnectionHandle>,
    game_server: Option<ConnectionHandle>,
    session_time: Option<SessionTime>,    // round 5: server time lives here
    it_is_2d_3d_example: bool,            // from the descriptor at creation (matchmaking game type)
    connection_status_visible: bool,      // today's NetSystem.connection_status_visible
    // + the score/team state it already has, connection UI state
}

struct SessionTime
{
    server_time_at_join_ms: u128,
    joined_at: Instant,
}
impl SessionTime { fn server_time_ms(&self) -> u128; }  // today's TimeSystem::get_server_time

pub enum SessionControllerMessage
{
    // existing: JoinedToSession, NewSessionStarted, SetScore, TeamWin … (now produced internally)
    // new, from actors (no timestamps: SessionController adds server time when building the packet):
    TryToReturnMyFlag,
    TryToGetOpponentsFlag,
    TryToGetScore,
    TryToGetMoveWBonus(/* .. */),
    SendToPlayers { message: NetMessageToPlayer, reliability: Reliability },
    SendToPlayer { message: NetMessageToPlayer, peer: ActorID, reliability: Reliability },
    ShowConnectionStatusUI,
}
// CommonActorsMessage gets IAmSessionController(ActorID), broadcast on start; actors that send
// network data store the id. MainPlayer gets PlayerMessage::ChangeYourId(ActorID).
```

`InputSystem` (shared, round 5): collects input once per tick; actors read it directly.

```rust
impl InputSystem
{
    pub fn actions(&self) -> &ActionsFrameState;   // computed once in pre_simulate (today: get_input() per call)
    pub fn mouse_delta(&self) -> Vec2;
    // existing: set_keyboard_input, set_mouse_button_input, add_mouse_delta, set_mouse_wheel_input,
    //           collect_gamepad_button_input, reset_input
    // removed: set_input_to_controlled_actors (InputMaster is gone)
}
```

### 21.10 Startup, initial world, splash, and `pending_root_world` [accepted] / [proposed]

- `Engine::new(window, EngineConfig { initial_world, .. })` creates the shared systems. `RenderScheduler::new` spawns the render thread, which **shows the splash immediately**.
- The initial descriptor loads, then `set_root_world` creates the first `World` and presents it.
- Later the main menu is just another `initial_world`. Between worlds no splash is needed: the current world runs while the next descriptor loads, then `ReplaceRootWorld` swaps instantly.

**Re-evaluation of `pending_root_world` (author's question).** The author's idea: since the render thread runs independently and shows the splash, `Engine::new` could load the initial descriptor and create the first world **synchronously**, and the field disappears. Claude's assessment:
- It works technically: the render thread keeps animating the splash while the main thread blocks.
- The catch is the **OS, not the engine.** `Engine::new` runs inside winit's `new_events(StartCause::Init)` on the main thread. While it blocks, the window's event queue isn't pumped:
  - **Windows** marks a window that doesn't process messages for ~5 s as "Not Responding", replaces it with a frozen ghost image (covering our animated splash), and may offer to close the program.
  - **macOS** shows the spinning beach-ball cursor after a few seconds.
  - Linux/Wayland compositors may also show "not responding" dialogs.
- A **first launch** compiles shaders (slow, often many seconds; see `AGENTS.md`). That's exactly when a splash matters, and exactly when the sync variant would show "Not Responding". (Today's `Engine::new` blocks the same way; there's just no splash to notice.)
- The async variant costs **one field and ~8 lines** (`poll_pending_root_world`), and reuses the same `spawn_blocking` + `JoinHandle` pattern actors use.
- [proposed] **Recommendation: keep `pending_root_world`.** If the author still prefers sync, it's a two-line change and doesn't affect the rest of the architecture.

### 21.11 `GlobalSettings` (done in round 5)

```rust
pub struct GlobalSettings
{
    pub user: UserSettings,               // saved to the user file
    pub connection: ConnectionSettings,   // compiled in; overridable only by the file next to the binary
    user_file: Option<PathBuf>,           // where save() writes `user`
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]                         // missing keys → defaults; unknown keys ignored
pub struct UserSettings
{
    pub mouse_sensitivity: f32,           // was PlayerSettings.mouse_sensivity: Arc<Mutex<f32>>
    pub master_volume: f32,               // N/M keys
    pub screen_resolution_scale: f32,     // i/O keys; initial output scale of the root world
    pub shadows_enabled: bool,            // P key
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ConnectionSettings
{
    pub matchmaking_server_url: String,
    pub ice_servers: Vec<String>,         // was bash_and_turn_servers
    pub turn_server_username: String,
    pub turn_server_credential: String,
}

impl GlobalSettings
{
    /// user:       <exe dir>/user_settings.json → <config dir>/slice4d/user_settings.json → create with defaults
    /// connection: embedded defaults (include_str!) ← overridden by <exe dir>/connection_settings.json if present
    pub fn load() -> GlobalSettings;
    pub fn save(&self);                   // writes `user` to user_file; logs on failure
}
```

- [accepted] User preferences only in the user file; connection data compiled in, overridable only next to the binary (so new releases can change servers / rotate TURN credentials).
- [accepted] Platform config directories: Linux `~/.config/slice4d/` (`$XDG_CONFIG_HOME`), Windows `%APPDATA%\Slice4D\`, macOS `~/Library/Application Support/Slice4D/`. [proposed] Via the small `dirs` crate (`dirs::config_dir()`).
- [accepted] `serde` with `derive` (new direct dependency; already in the tree through other crates).
- "Next to the binary" = `std::env::current_exe()?.parent()`, not the current directory (§14.3).
- [proposed] File names: `user_settings.json` and `connection_settings.json` (two files, because they have different lifecycles). Directory name `slice4d` / `Slice4D` following each platform's casing habit.
- MainPlayer gameplay values become constants (§14.2); unused fields are deleted.

### 21.12 Ownership summary

```
Slice4DShooter (winit)
└── Engine
    ├── root_world: Option<World>
    │     ├── actors: Actors ── ActorWrapper ── (some actors) ── nested World (owned, simulated in tick)
    │     ├── messenger (message pump), physics, time
    │     └── render: WorldRenderSystem ── Arc<WorldRenderShared> ──┐ (atomics)
    ├── pending_root_world: JoinHandle<WorldDescriptor>             │
    ├── shared: EngineSharedSystems                                  │
    │     ├── gpu: GpuContext (device, queue, layouts)
    │     └── render: RenderScheduler { GpuContext clone, Sender } ──► render thread: RenderLoop
    │                                       └── slots: WorldSlot { Weak<WorldRenderShared> ◄┘, WorldGpu }
    └── window
WorldDescriptor (Arc): held by whoever loaded it (an actor, the engine during startup, later the Resource System cache);
                       not held by World.
```

---

## 22. Key operations (pseudo-code), revision 3 [accepted]

```rust
// 1. Create the Engine (main_loop.rs, StartCause::Init)
let engine = pollster::block_on(Engine::new(Some(window), EngineConfig { initial_world: "capture_the_flag_map".into(), .. }));

impl Engine
{
    pub async fn new(window: Option<Window>, config: EngineConfig) -> Engine
    {
        let window = window.map(Arc::new);
        let shared = EngineSharedSystems::new(window.clone(), &config).await;   // render thread starts, splash visible
        let gpu = shared.gpu.clone();
        let name = config.initial_world.clone();
        let pending = shared.runtime.spawn_blocking(move || WorldDescriptor::load(&name, gpu));
        Engine { root_world: None, pending_root_world: Some(pending), shared, window, quit_requested: false }
    }

    // 2. Engine tick
    pub fn tick(&mut self)
    {
        self.shared.pre_simulate();
        self.poll_pending_root_world();

        if let Some(world) = &mut self.root_world
        {
            world.simulate(&mut self.shared);
            for message in world.take_outer_world_messages()
            {
                self.handle_outer_world_message(message);
            }
        }

        self.shared.post_simulate();
    }

    fn poll_pending_root_world(&mut self)
    {
        if self.pending_root_world.as_ref().is_some_and(|h| h.is_finished())
        {
            let handle = self.pending_root_world.take().unwrap();
            let descriptor = self.shared.runtime.block_on(handle).unwrap();
            self.set_root_world(Arc::new(descriptor));
        }
    }

    // 3. Replace the root world (also used for the initial world)
    fn set_root_world(&mut self, descriptor: Arc<WorldDescriptor>)
    {
        let config = WorldConfig {
            render: self.window.as_ref().map(|w| WorldRenderConfig {
                resolution: w.inner_size().into(),
                scale: self.shared.global_settings.user.screen_resolution_scale,
                render_rate: 1,
            }),
        };
        let new_world = World::new(&descriptor, config, &mut self.shared);   // cheap; descriptor not kept
        if let Some(render) = &self.shared.render
        {
            render.present_world(new_world.output_texture_id());
        }
        let _old = self.root_world.replace(new_world);   // old world dropped here: its slot dies
    }

    fn handle_outer_world_message(&mut self, message: OuterWorldMessage)
    {
        match message
        {
            OuterWorldMessage::QuitApplication => self.quit_requested = true,
            OuterWorldMessage::ReplaceRootWorld(descriptor) => self.set_root_world(descriptor),
            OuterWorldMessage::WorldFinished => {}
            OuterWorldMessage::Specific(_) => log::warn!("engine ignores specific outer-world messages"),
        }
    }

    pub fn on_window_resized(&mut self, size: PhysicalSize<u32>)
    {
        if let Some(render) = &mut self.shared.render { render.resize_surface(size); }
        if let Some(world) = &self.root_world { world.set_output_resolution(size.into()); }
    }
}

// 4. World::new
impl World
{
    pub fn new(descriptor: &WorldDescriptor, config: WorldConfig, shared: &mut EngineSharedSystems) -> World
    {
        let physics = PhysicsSystem::new(descriptor);                      // copies static colliders
        let render = match (&shared.gpu, &mut shared.render, &config.render)
        {
            (Some(gpu), Some(scheduler), Some(render_config)) =>
                Some(WorldRenderSystem::new(descriptor, render_config, gpu, scheduler)),  // creates GPU objects, registers
            _ => None,
        };
        let mut world = World { actors: Actors::new(), messenger: Messenger::new(), physics,
                                time: WorldTimeSystem::new(), render };

        let (main_actor, other_actors) = descriptor.create_actors();       // descriptor knows JSON, World doesn't
        world.messenger.add_main_actor(main_actor, &mut world.actors, &world.physics, &world.time, shared);
        world.messenger.add_actors(other_actors, &mut world.actors, &world.physics, &world.time, shared);
        world.messenger.start_world(&mut world.actors, &world.physics, &world.time, shared);   // on_world_started
        world.messenger.process_messages(&mut world.actors, &world.physics, &world.time, shared);
        world
    }

    // 5. simulate
    pub fn simulate(&mut self, shared: &mut EngineSharedSystems)
    {
        self.time.begin_simulate(&shared.time);

        let mut ctx = WorldContext { messenger: &mut self.messenger, physics: &self.physics,
                                     time: &self.time, main_actor_id: self.actors.main_actor_id() };
        for (_, actor) in self.actors.iter_mut() { actor.tick(&mut ctx, shared); }   // actors read shared.input

        self.messenger.process_messages(&mut self.actors, &self.physics, &self.time, shared);
        self.physics.process_physics(&mut self.actors, self.time.delta(), &mut self.messenger);
        self.messenger.process_messages(&mut self.actors, &self.physics, &self.time, shared);

        if let Some(render) = &mut self.render
        {
            render.update(&self.actors, &self.time, &shared.global_settings);
        }
    }

    // 6. Render control (atomics, callable any time)
    pub fn set_render_rate(&self, rate: u32)
    {
        if let Some(r) = &self.render { r.shared.render_rate.store(rate, Ordering::Relaxed); }
    }
    pub fn set_output_resolution_and_scale(&self, resolution: UVec2, scale: f32)
    {
        if let Some(r) = &self.render { r.shared.output.store(pack(resolution, scale), Ordering::Relaxed); }
    }
}

// 7. A nested world owned and simulated by an actor (illustration only; the tutorial screen is later work)
pub struct TutorialScreen
{
    id: Option<ActorID>,
    loading: Option<tokio::task::JoinHandle<WorldDescriptor>>,   // later: a Resource System handle
    demo_world: Option<World>,
    tick_counter: u32,
}

impl Actor for TutorialScreen
{
    fn on_added_to_world(&mut self, _world: &mut WorldContext, shared: &mut EngineSharedSystems)
    {
        let gpu = shared.gpu.clone();
        self.loading = Some(shared.runtime.spawn_blocking(move || WorldDescriptor::load("map_2d_3d", gpu)));
    }

    fn tick(&mut self, world: &mut WorldContext, shared: &mut EngineSharedSystems)
    {
        if self.loading.as_ref().is_some_and(|h| h.is_finished())
        {
            let descriptor = shared.runtime.block_on(self.loading.take().unwrap()).unwrap();
            let config = WorldConfig { render: Some(WorldRenderConfig {
                resolution: UVec2::new(512, 512), scale: 1.0, render_rate: 2 }) };
            self.demo_world = Some(World::new(&descriptor, config, shared));
        }

        if let Some(demo) = &mut self.demo_world
        {
            // owner → world
            if shared.input.actions().activate.is_action_just_pressed()
            {
                demo.send_broadcast_message(MessageType::SpecificActorMessage(/* StartDemo */));
            }

            // simulate every 2nd tick (the world's own delta keeps real-time speed)
            self.tick_counter += 1;
            if self.tick_counter % 2 == 0 { demo.simulate(shared); }

            // LOD
            demo.set_output_scale(if /* player far away */ false { 0.5 } else { 1.0 });

            // world → owner
            for message in demo.take_outer_world_messages()
            {
                match message
                {
                    OuterWorldMessage::WorldFinished => { self.demo_world = None; }   // unregisters automatically
                    OuterWorldMessage::QuitApplication | OuterWorldMessage::ReplaceRootWorld(_) =>
                        world.messenger.send_to_outer_world(message),               // forward outward
                    OuterWorldMessage::Specific(_) => {}
                }
            }
        }
    }
    // 8. The world's texture: demo.output_texture_id() is plain data (valid in headless mode too).
    //    Displaying it: issue issues/programming/in-world-4d-screens/ (resolved on the render thread).
}

// 9. Changing the own id (round 5)
// SessionController, on NetSystemIsConnectedAndGetNewPeerID(new_id):
world.messenger.send_direct_message(world.main_actor_id, Message {
    from: self.id.unwrap(), remote_sender: false,
    message: MessageType::SpecificActorMessage(SpecificActorMessage::PlayerMessage(PlayerMessage::ChangeYourId(new_id))),
});
// MainPlayer::recieve_message:
world.messenger.send_command(Command { sender: self.id.unwrap(), command_type: CommandType::ChangeMyId(new_id) });
// Messenger::execute_command: actors.change_id(command.sender, new_id), broadcast IWasChangedMyId (as today),
//                             update main_actor_id if it was the main actor.

// 10. Replacing the root world from inside (FinalTrigger, §20.10)
world.messenger.send_to_outer_world(OuterWorldMessage::ReplaceRootWorld(Arc::new(descriptor)));

// 11. Shutdown
// winit: CloseRequested (or engine.quit_requested()) → event_loop.exit() → Slice4DShooter dropped → Engine dropped:
//   root_world dropped (actors, nested worlds, Arc<WorldRenderShared>)
//   pending_root_world dropped (a running spawn_blocking task finishes on its own; its result is discarded)
//   shared dropped: RenderScheduler::drop sends Shutdown + joins the render thread; then net/audio/ui; runtime last
//   window dropped on the main thread
// GlobalSettings::save() runs on settings changes (key release) and once on shutdown.
```

---

---

# Part IV — Technical specification and implementation plan

## 24. Technical specification (done in round 6, final)

This section is the implementation contract. It lists every agreed change from the current code (Part II §2) to the target architecture (Part III §21–§22). Rationale is in Part I; history in Part II.

### 24.1 Scope

**In scope (this refactoring):**
1. `EngineSharedSystems` + `Engine { root_world, pending_root_world, shared, window }`; `HeadlessEngine` deleted.
2. `World { actors, messenger, physics, time, render }` with `World::new(&WorldDescriptor, WorldConfig, &mut shared)` and `World::simulate(&mut shared)`.
3. `Actors` (former `World` actor container), `Messenger` (former `EngineHandle` + the whole message pump + command execution + outer-world mailbox).
4. New actor callback signature: `(&mut self, world: &mut WorldContext, shared: &mut EngineSharedSystems)`; `on_level_loaded` → `on_world_started`; `send_boardcast_message` → `send_broadcast_message`.
5. `impl Actor for ActorWrapper` generated by `macro_rules!` (same enum, static dispatch).
6. `WorldDescriptor` (former `Level`) with actor descriptions and `create_actors()`; `WorldDescriptor::load(name, Option<GpuContext>)` (blocking; async via `spawn_blocking`).
7. `OuterWorldMessage` + root-world replacement via `ReplaceRootWorld`; `LoadNewLevelSync/Async`, `PreloadNewLevelAsync`, `World.preloaded_levels`, `level_async_load_handler` deleted; `FinalTrigger` migrated.
8. `GlobalTimeSystem` (shared) + `WorldTimeSystem` (per world, own delta); `TimeSystem` deleted; server time → `SessionController`.
9. `InputMaster`, `ControlledActor::get_input_master`, `set_input_to_controlled_actors` deleted; actors read `shared.input`; `InputSystem` without `Arc<Mutex<…>>`.
10. `EffectsSystem` deleted; `spawn_wave` becomes a free function.
11. `PlayerSettings` and `Settings` deleted; gameplay values → constants; `GlobalSettings { user, connection }` with `serde`, platform config dir (`dirs`), `user_settings.json` / `connection_settings.json`, save on change.
12. `NetSystem` → transport with `ConnectionHandle`s (WebSocket, WebRTC); all session logic, matchmaking request, `process_message`, connection UI and session time → `SessionController`; other actors send network data to `SessionController` (V1); `IAmSessionController(id)` discovery; `ChangeYourId` / `ChangeMyId`.
13. Rendering: `GpuContext` (shared field); `RenderScheduler` with a dedicated OS render thread, inbox channel, `WorldSlot`s with `Weak<WorldRenderShared>`, reverse registration order, no `Mutex<WorldGpu>`, atomics for render rate and packed resolution/scale, `WorldTextureId`, fixed world output format (sRGB-preserving), `SplashScreen`, UI renderer re-bound to the presented world, clean shutdown.
14. Nested-world API: `set_render_rate`, `set_output_resolution`, `set_output_scale`, `set_output_resolution_and_scale`, `output_texture_id`, `send_broadcast_message` / `send_direct_message` (sender 0), `take_outer_world_messages`.
15. Window-level hotkeys move to the engine level and operate on `shared` (`i`/`O` → root world output scale, `P` → `shadows_enabled`, `J`/`K` sensitivity, `N`/`M` volume), persisting user settings.
16. Delete the untracked WIP files.
17. Update `AGENTS.md` to the new architecture when done.

**Out of scope:** see Part I §I.5 (in-world screens, Resource System, authoritative server, UI/Input rewrites, removing `SpawnControlledActor`, FIFO, `Option<ActorID>` cleanup, error-handling overhaul, misspellings). Also: no protocol changes (`client_server_protocol.rs`, `matchmaking_server_protocol`) and no server changes. **Don't bump the version** (auto-update!).

### 24.2 Behaviour that must stay identical

- Startup into `capture_the_flag_map` (or `-l <level>`), `-b` backend, ~~`-m observer` mode~~ ([rejected] round 7 point 4: `-m` removed, `game_mode` was never read).
- Player movement/physics feel: same constants, same delta for the root world (same 0.35 s clamp → 1/60).
- All weapons, holes, flags, bonuses, scanner, HUD, sounds.
- Render: same image (colours!), same ~125 FPS pacing target, "no frame queueing", same `i`/`O` 5% steps within 0.1–1.0, `P` shadows, minimap pass.
- Network: press `P` to connect, matchmaking with version/game type, joining, other players' dolls, all relayed messages, score/flags, team win, disconnect/reconnect UI and timers.
- Hotkeys `Esc`/`Enter` (fullscreen/cursor), `Y`/`H` (open browser when the tutorial window is visible), `J`/`K`, `N`/`M`.

**Accepted behaviour changes** (consequences of agreed decisions):
- Network is available only in worlds that contain a `SessionController` (V1). Today that's the CTF map (the main game). Obstacle-course levels and the 2D/3D example map no longer react to `P` by connecting. (The 2D/3D example would need a session controller in its map to be networked; it's going to be a nested, offline demo anyway.)
- User preferences persist across launches (new); a local file can no longer override gameplay constants (closes a cheat vector).
- A splash is shown while the initial world loads (new).

### 24.3 Target module layout

```
src/game_client_main.rs             entry point only (args, logger, allocator, auto-updater, Engine::new(headless).run(start_level, graphic_backend))
src/engine.rs                       Engine, EngineConfig (mod declarations)
src/engine/main_loop.rs             Engine::run (+ private run_headless) + private MainLoop (winit ApplicationHandler: window, hotkeys, events → engine) (round 7: moved from src/main_loop.rs)
src/engine/transform.rs             Transform (round 7: moved from src/transform.rs)
src/engine/shared_systems.rs        EngineSharedSystems (declares the modules below; round 7: moved into shared_systems/)
src/engine/shared_systems/global_settings.rs   GlobalSettings, UserSettings, ConnectionSettings
src/engine/shared_systems/time.rs   GlobalTimeSystem (+ WorldTimeSystem in world/time.rs)
src/engine/shared_systems/input.rs  InputSystem (no InputMaster)
src/engine/shared_systems/net.rs    NetSystem transport only
src/engine/shared_systems/audio.rs, ui.rs      unchanged except signatures
src/engine/shared_systems/render.rs (+ render/)  GpuContext, RenderScheduler, RenderLoop, WorldRenderSystem, WorldGpu
                                    (WorldRenderSystem stays here: layout exception, see §24.9 round 7 points 2–3)
src/engine/world.rs                 World, WorldConfig, WorldRenderConfig, WorldContext, OuterWorldMessage
src/engine/world/actors.rs          Actors
src/engine/world/messenger.rs       Messenger, Command, CommandType
src/engine/world/time.rs            WorldTimeSystem
src/engine/world/world_descriptor.rs WorldDescriptor (former level.rs)
src/engine/world/static_object.rs   unchanged
src/engine/world/physics.rs (+ physics/)  PhysicsSystem (per world; takes &mut Actors) (round 7: moved from src/engine/)
src/engine/world/effects.rs         spawn_wave (free fn over the world's Messenger) (round 7: moved from src/engine/)
src/actor.rs                        Actor, ControlledActor, actor_wrapper! macro, messages
src/actor/session_controller.rs (+ session_controller/)   session logic incl. former net.rs parts
src/actor/main_player/constants.rs  former PlayerSettings gameplay values
```

Module names may differ slightly where the existing layout makes another name more natural; any deviation is noted in §24.9.

### 24.4 Implementation stages

Each stage ends with `cargo check --bin game-client` (and `cargo build --release --bin game-client` at the end) passing. Warnings count should not grow meaningfully.

**Stage 1 — structure, actors, settings, time, input, descriptor**
1. `GlobalSettings` (`serde` + `dirs` deps), loading order and save; constants in `main_player/constants.rs`; delete `PlayerSettings`, `Settings`, `settings2.json` usage. Keep the embedded `src/assets/settings/settings.json` values as the source of compiled-in defaults (split into user/connection parts).
2. `GlobalTimeSystem` from `TimeSystem` (minus server time); `WorldTimeSystem`.
3. `InputSystem`: delete `InputMaster` usage paths; `actions()` computed once per tick; remove `Arc<Mutex<…>>` from the actions table if it's main-thread-only (verify: no clones to other threads).
4. `EngineSharedSystems` (temporarily still holding the old `RenderSystem` and old `NetSystem` until stages 2–3).
5. `Actors`, `Messenger` (pump + commands + outer mailbox; LIFO `pop()` as today), `WorldContext`, `World::new/simulate`.
6. `Actor` trait new signatures; `actor_wrapper!` macro; migrate all actors (mechanical: `engine_handle` → `world.messenger`, `physic_system` → `world.physics`, `audio_system` → `shared.audio`, `ui_system` → `shared.ui`, `time_system` → `world.time`/`shared.time`, `delta` → `world.time.delta()`, `effects_system` → free function, `player_settings.X` → constants, input via `shared.input.actions()`).
7. `WorldDescriptor` from `Level` (actor JSON stored, `create_actors()`, `load(name, gpu)`), `PhysicsSystem::new(&descriptor)`.
8. `Engine` with `root_world`, `pending_root_world`, `OuterWorldMessage` handling, `set_root_world`; `FinalTrigger` migrated; delete `LoadNewLevel*`/`PreloadNewLevelAsync`.
9. Transitional (removed in stage 2): `CommandType::NetCommand` collected by the `Messenger` into a world-level outbox that the engine forwards to the old `NetSystem`; the old `NetSystem::tick` writes into the root world's `Messenger`.

**Stage 2 — network**
1. `NetSystem` transport (§21.9): handles, WebSocket task, WebRTC socket, states, receive buffers, `local_peer_id`.
2. `SessionController`: state machine, matchmaking request/response, `ServerMessage` decoding, `process_message` (moved as-is, adapted to `world.messenger` / `shared.audio`), connection UI, player list, `SessionTime`, timestamps for outgoing messages, `IAmSessionController(id)` broadcast, `ChangeYourId`.
3. Actors that sent `NetCommand`s send `SessionControllerMessage`s instead (main_player, player_for_2d_3d_example, holegun, obstaclesgun, machinegun, players_doll, obstacle_course_free_movement_player, shotgun, flag); `ShowConnectionStatusUI` → message to `SessionController`.
4. Delete `CommandType::NetCommand` and the transitional outbox.

**Stage 3 — rendering**
1. `GpuContext::new` (from `Renderer::new`'s instance/adapter/device/layout code); `EngineSharedSystems.gpu`.
2. `RenderScheduler` + `RenderLoop` on a `std::thread` (inbox, pacing, `QueueEmpty` check, surface errors handling as today, reverse-order slot encoding, presented world → upscale → UI → submit → present, splash when none, `Shutdown` + `join` in `Drop`).
3. `WorldRenderSystem` (main thread: `RenderData`, buffer clones, `update()` = today's `send_data_to_renderer`) and `WorldGpu`/`OutputTexture` (render thread).
4. Fixed `WORLD_OUTPUT_FORMAT` for output textures and world pipelines; upscale pipeline targets the surface format. Verify identical colours (keep an sRGB format for the output texture, matching today's surface format family).
5. `UIRenderer::bind_world_buffers` on `PresentWorld`.
6. Render rate / packed resolution+scale atomics; texture recreation + generation on the render thread; `i`/`O` → `set_output_scale` on the root world.
7. Delete the old `RenderSystem`, `Renderer`'s `Arc<Mutex<…>>`, `main_raymarch_shader_pipeline` mutex, `MainShaderRenderPipelineBuilderKit`.

**Stage 4 — nested-world API, cleanup, docs**
1. Inward messages (sender 0), `take_outer_world_messages`, `output_texture_id`, render control API (all present and compiled even if unused by the game today).
2. Headless path compiles (`window: None` → `gpu`/`render` `None`).
3. Delete WIP files (`src/engine/shared_systems/`, `src/engine/world/{actors,effects,geometry,level_descriptor,messenger,physics,render}.rs` + dirs, `src/engine/main_loop.rs`, `src/engine/transform.rs`) before/while creating the real modules with the same names (stage 1 starts by deleting them to avoid confusion).
4. `AGENTS.md`: architecture section rewritten for the new structure; issue status updated.

### 24.5 Acceptance criteria

- `cargo check` and `cargo build --release --bin game-client` succeed.
- The client starts, shows the splash, then the CTF map; the player can move, jump, shoot all weapons, use the scanner; HUD and sounds work.
- `i`/`O`, `P`, `J`/`K`, `N`/`M` work; after restart, sensitivity/volume/scale/shadows are restored from `user_settings.json` in the platform config dir.
- Online: `P` connects through matchmaking to a game server; two clients see each other, shooting/holes/flags/score sync; disconnect shows the right UI and recovers. *(Requires a reachable matchmaking + game server; to be verified by the author if the agent can't run servers.)*
- Window resize and fullscreen toggling keep rendering correct.
- Closing the window exits cleanly (render thread joined; no panic on exit).
- No `Mutex` on the render path; render loop not on tokio.
- The nested-world API exists and compiles: a world can be created from a descriptor by an actor, simulated, rendered into its own texture with a controllable rate/resolution/scale.

### 24.6 Verification plan

There are no automated tests. Verification is by building and running:
1. `cargo check` after every stage.
2. A short local run of the client (`cargo run --release --bin game-client`) to confirm startup, splash → CTF map, and a clean exit (logs via `RUST_LOG=info`).
3. Colour check: compare with the pre-refactoring build (screenshots) if possible.
4. Network: a local `game_server` + `matchmaking_server` (with a local `connection_settings.json` next to the binary pointing at localhost) and two clients, if feasible in the agent's environment; otherwise listed for the author to verify.

### 24.7 Notes on specific migrations

- **`SessionController` id discovery:** `SessionController` broadcasts `CommonActorsMessage::IAmSessionController(id)` in `on_world_started` (and after its id changes). Actors that send network data store `Option<ActorID>`; before it's known, network messages are dropped (same as being offline).
- **Main player id change:** `SessionController` → `PlayerMessage::ChangeYourId(new)` to `world.main_actor_id` → player sends `CommandType::ChangeMyId(new)` → `Messenger` changes the id (today's `World::change_actor_id` logic incl. collision handling and `IWasChangedMyId` broadcast) and updates `Actors::main_actor_id`.
- **Server time stamps:** outgoing messages that carried `get_server_time()` are sent to `SessionController` without the time; it fills in `SessionTime::server_time_ms()` when building the protocol message.
- **`PlayerDoll::new(.., player_settings, ..)`** takes constants instead.
- ~~**Observer mode** (`-m observer`) keeps working through `EngineConfig.game_mode` (passed where it's used today).~~ [rejected] round 7 point 4: `game_mode` was never read by anything; removed with `EngineConfig`.
- **`ShowConnectionStatusUI`**: message to `SessionController` (sets `connection_status_visible`).
- **`RemoveAllHolesAndEffects`**: stays a world command, sent by `SessionController`.
- **`it_is_2d_3d_example`**: descriptor field; given to `SessionController` and `PlayerDoll` at creation.

### 24.8 Risks

- Message-order sensitivity: kept LIFO, so no change.
- Colours with the fixed output format (§20.6).
- `SessionController` network migration (largest behavioural risk; manual two-client test).
- Render-thread lifetime and drop order (window destroyed on the main thread).
- Diff size: every actor changes; the macro keeps `ActorWrapper` consistent.

### 24.9 Implementation log

Deviations from §21/§24.4, problems found, decisions taken while coding (2026-11-12).

**Order of work.** Stages 1–3 were done in **one pass** instead of three: transitional adapters (old `NetSystem` fed by the new `Messenger`, old `RenderSystem` fed by `Actors`) would have been almost as much code as the real migration and thrown away right after. Stage 4 (nested-world API) is part of the same pass. `cargo check` was used as the guide after the pass; the WIP files were deleted at the start.

**Networking.**
- `send_to_session_controller(messenger, from, message)` **broadcasts** the message instead of a direct message with an `IAmSessionController` discovery protocol (§21.9 / §24.7 planned discovery). This is the fallback already described in §13.4. Reasons: no id bookkeeping in 9 files, no lost messages before discovery, and devices (weapons, which aren't actors) can use it too. Switching to direct delivery later means changing this one function. Cost: one extra match per actor per network message (negligible).
- `SessionControllerMessage` got the outgoing variants `TryToGetOpponentsFlag`, `TryToGetScore`, `TryToReturnMyFlag`, `TryToGetMoveWBonus(index)` (stamped with the session time by `SessionController`), `SendToServer`, `SendToPlayers(msg, Reliability)`, `SendToPlayer(msg, peer, Reliability)`, `SendPlayerDollState(transform, input, velocity)` (stamped), `ShowConnectionStatusUI`.
- The ~60 former `NetCommand` sends were converted mechanically (script with a bracket parser), then re-indented.
- The connection state machine, the matchmaking request (`get_game_server_url`), `ServerMessage` decoding and `process_message` (unchanged logic) moved to `actor/session_controller/net_session.rs` (`NetSession`, owned by `SessionController`). Decoded server events are still broadcast with sender id `0` (as before), so `SessionController` and all other actors receive them exactly as before.
- `NetSystem` (transport): WebSocket connections run in a tokio task (reader) + a writer task with unbounded channels; WebRTC uses `WebRtcSocket` as before. Both channels' packets go into one receive buffer (before, packets on the unreliable channel that are only expected on the reliable one were just logged as errors; now they're processed — harmless).
- `PlayerMessage::ChangeYourId(new)` → `CommandType::ChangeMyId(new)` implemented as agreed.

**Rendering.**
- `GpuContext` also holds the shared resources every world uses: the sky box texture + sampler (decoding 6 PNGs per `World::new` would make it slow), the upscale bind-group layout + sampler, the full-screen and minimap vertex/index buffers. It doesn't store `Instance`/`Adapter` (not needed after creation). Layouts are two fields (`raymarch_bind_group_layout_0/1`) instead of an array.
- `WorldDescriptor::load(name, Option<GpuContext>)`; `WorldRenderSystem::new(descriptor, config, &mut shared)` returns `None` when headless or when the descriptor has no GPU part.
- `WORLD_OUTPUT_FORMAT = Rgba8UnormSrgb`. sRGB encode on write + decode on sample is an exact inverse pair, so the image is the same as with the old surface-format target, whether or not the surface format is sRGB.
- `UIRenderer::bind_world_buffers(Option<ScannerWorldBuffers>)` rebuilds the scanner bind groups when the presented world changes; the UI is not drawn over the splash.
- The splash is `media/slice_4d_shooter_poster.png` (embedded, decoded on the render thread). Static for now; the render thread could animate it later.
- Not implemented yet (not needed by the game now): a render-thread resolver `WorldTextureId → TextureView` for consumers of nested-world textures. `WorldTextureId` and the texture generation counter exist; the resolver comes with `issues/programming/in-world-4d-screens/`.

**Input.** The `Arc<Mutex<…>>` in `InputSystem` wasn't for threads: several buttons (keyboard + gamepad) share one action. Replaced with `actions: Vec<(ButtonActions, Action, i32)>` + `actions_table: HashMap<SomeButton, usize>`. `ActionsFrameState` is computed once per tick in `pre_simulate` (`shared.input.actions()`).

**Settings.** `serde` (derive) and `dirs` added to `Cargo.toml` (`Cargo.lock` gained only these). `settings.json` and `settings2.json` were replaced by `src/assets/settings/connection_settings.json` (embedded). `user_settings.json` is created in the platform config dir on first launch. `master_volume` is persisted too. `J`/`K`, `N`/`M` save on key release; `i`/`O`, `P` save immediately; the app saves once more on exit.

**Actors.** The signature change was applied by a script that rewrote the trait methods and inserted bindings for the old parameter names still used in the bodies (`let messenger = &mut *world.messenger;`, `let audio_system = &mut shared.audio;`, `let delta = world.time.delta();`, …). This keeps the actor bodies (≈21k lines) unchanged. `PlayerSettings` uses were replaced by constants (`main_player/constants.rs`); functions that read the mouse sensitivity now take `mouse_sensitivity: f32` (callers pass `shared.global_settings.user.mouse_sensitivity`). `actor.rs` went from 1545 to ~400 lines thanks to `actor_wrapper!`.

**Other.** `FinalTrigger` preloads the next descriptor with `spawn_blocking` and sends `ReplaceRootWorld` when ready (sync mode blocks on the handle). `Engine::new` stays `async`. The map JSON of the CTF map is named `test_level` (shows in the log as the root world name).

**Verification done by the agent.**
- `cargo check` / `cargo build --release --bin game-client`: OK (702 warnings, the baseline had 876).
- Ran the release client on the author's X11 desktop several times: splash → CTF map loads, HUD and scanner render, colours look as before (screenshot checked), no errors in the log.
- Clean shutdown: closing the window (`WM_DELETE_WINDOW`) exits with code 0 in ~0.2 s; the log shows the render thread stopping and joining.
- Network: during one run, synthetic key events (xdotool) happened to trigger "connect", and the client went through the whole chain **against the production servers**: matchmaking → WebRTC game server → "Joined to game session". So the new transport + `SessionController` work end to end. (A test client was briefly connected to the live server; it disconnected when the process was killed.)
- `~/.config/slice4d/user_settings.json` was created on the author's machine by these runs.
- **Performance comparison** (same machine, same conditions, release builds, CTF map, 12 s, full-screen window): render loop average frame time **old build 73.96 ms** (min 69.7, max 89.4) vs **new build 73.07 ms** (min 71.2, max 112.0). No regression (this machine's GPU is slow at full resolution; the numbers are only comparable with each other).
- **Shutdown time** after the close request: old build **9.17 s**, new build **0.21 s** (the old render task on tokio wasn't stopped explicitly).

**Round 7 (2026-11-13), the author's feedback, point 1: no unnecessary allocations.**
- [accepted] Author's rule (now a core principle in `AGENTS.md`): wherever an allocation or reallocation can be avoided, avoid it, even with a fast allocator.
- `Messenger::take_outer_world_messages` used `std::mem::take`: the mailbox was replaced by an empty `Vec`, so the next message reallocated it. Now it's an iterator that `pop()`s one message per step (`std::iter::from_fn`), plus `pop_outer_world_message()`. The engine uses the `pop` form in a `while let` loop, because handling a message needs `&mut self` (replacing the root world), which an iterator borrowing the world wouldn't allow. Same API on `World`.
- Similar places found and changed (interface-only changes):
  - `NetSystem::receive` also used `mem::take` → returns `std::vec::Drain` (keeps the buffer's capacity). `drain(..)` rather than `pop()` here because **packet order matters** (e.g. spawn before state updates).
  - `PhysicsSystem::sphere_cast_on_dynamic_colliders` allocated a `Vec<Hit>` per call (several calls per tick per player) → returns a lazy iterator (`filter_map` over the player forms). All callers only iterate.
  - WebSocket send copied the packet (`packet.to_vec()`) → the `Box<[u8]>` is moved into the message without copying. Receive converts `Bytes` → `Vec` without copying when the buffer isn't shared.
  - Removed the unused `Actors::ids()` (returned a `Vec`).
- Found but **not** cheap (needs restructuring, not just an interface change; left for a separate decision): `PhysicsSystem::process_physics` builds three `Vec`s of references (`kinematic_colliders`, `areas`, `dynamic_colliders`) every tick. Reusing them needs a different data layout (e.g. indices instead of `&mut` references), because a buffer of references can't be stored across ticks.
- Checked and fine: broadcast messages are cloned per receiving actor, but the message types contain no heap data (plain copies); per-frame render buffers are cleared and reused; actor/level creation allocations happen once.

**Round 7, point 2 (2026-11-13): shared systems in their own folder.** The author asked whether the shared-system modules should live in a `shared_systems/` folder for readability; Claude agreed (the file layout then mirrors the architecture: `shared_systems/` ↔ `world/`). Moved `audio`, `global_settings`, `input` (+ `input/`), `net`, `render` (+ `render/`), `time`, `ui` from `src/engine/` to `src/engine/shared_systems/`; `shared_systems.rs` declares them. Paths: `crate::engine::X` → `crate::engine::shared_systems::X`; relative `include_str!`/`include_bytes!` paths got one more `../`. `render/` keeps `world_render_system.rs` (the per-world render part), because it shares the render-thread types (`WorldGpu`, slots) with the scheduler. ~~`physics` (per world) and `effects` (a helper) stay in `src/engine/` for now (see the open question in the reply / next round).~~ [rejected] superseded by point 3: both moved to `world/`.

**Round 7, point 3 (2026-11-13): every module where it logically belongs.** The author confirmed the proposal and stated the general principle: everything that belongs to shared systems goes to `shared_systems/`, everything that belongs to a world goes to `world/`, everything that belongs to the engine as a whole goes to `engine/`; exceptions are documented with their reason. Done:
- `src/engine/physics.rs` (+ `physics/`) → `src/engine/world/physics.rs` (+ `world/physics/`): `PhysicsSystem` is per world.
- `src/engine/effects.rs` → `src/engine/world/effects.rs`: `spawn_wave` works with the world's `Messenger`.
- `src/transform.rs` → `src/engine/transform.rs`.
- `src/main_loop.rs` → `src/engine/main_loop.rs` (`pub mod main_loop` in `engine.rs`); `src/game_client_main.rs` keeps only the entry point (args, logger, global allocator, WinSparkle, `MainLoop::run`). The two commented-out alternative entry points that still exist (`*_without_autoupdate`, `*_with_windows_command_line`) got the same `mod`/`use` change.
- Paths rewritten by a script over `use` trees (`crate::engine::{physics,effects}` → `crate::engine::world::…`, `crate::transform` → `crate::engine::transform`, including inside grouped `use crate::{engine::{…}, transform::…}`). None of the moved files use `include_str!`/`include_bytes!`, so no asset paths changed.
- No new exceptions: none of these modules needed internal types made public. The only exception stays `world_render_system.rs` (point 2); it is now also documented by a comment at its `mod` declaration and in `AGENTS.md` (core principle 5, which also states the layout rule).
- Check: `cargo build --release --bin game-client` OK (no errors, no new warnings in the moved code), `cargo test` OK (0 tests, as before), client run: map loaded, exit code 0, render thread stopped cleanly.

**Round 7, point 4 (2026-11-13): the main loop inside the Engine, no crutch arguments.** The author: the entry point should create `Engine::new()` and call `engine.run(...)` (the main loop belongs inside the engine; `MainLoop::new` + `run(...)` with a list of flags was a quick test version). All odd arguments go: `with_ui_renderer` and `disable_net_system` were crutches from a time without room for refactoring; `game_mode` was never read and is removed entirely. `run` takes only `start_level` and `args.graphic_backend`; `Args` keeps `graphic_backend` (+ `specific_level`) so that more can be added later. (The alternative `*_main.rs` deletions in the working copy were made by the author himself.) Done:
- `src/game_client_main.rs`: `let engine = Engine::new(args.headless); engine.run(start_level, args.graphic_backend);` (no `pollster` here any more).
- `Engine::new(headless)` (sync) creates `EngineSharedSystems::new(headless)` without GPU/rendering and without a window: winit 0.30 lets a window be created only inside the running event loop. `impl Engine { pub fn run(self, initial_world, graphic_backend) }` lives in `engine/main_loop.rs`: creates the `EventLoop`, runs a private `MainLoop { engine, initial_world, graphic_backend }` (`ApplicationHandler`; the startup parameters are needed only at `StartCause::Init`) and saves user settings after the loop exits. At `Init`: create the window → `Engine::start(window, initial_world, backend)` (private, visible to the child module) → `shared.create_gpu_and_render(window, backend)` (`pollster::block_on(GpuContext::new)` + `RenderScheduler::new`), spawn the initial world loading, `time.init()`. `Slice4DShooter` and the old `MainLoop::new/run` are gone; `EngineConfig` is deleted.
- Removed: `EngineConfig { with_ui_renderer, disable_net_system, game_mode, … }`; `RenderScheduler::new(.., with_ui_renderer)` (`ui_renderer` is now a plain `UIRenderer`, not `Option`); `NetSystem::new(.., disabled)` with its "disabled" branch, and the `DisconnectReason::ClosedLocally` / `Transport::Closed` variants that only that branch created; `AudioSystem::new` is now sync (it never awaited anything; it keeps its `headless` parameter, see below); `GameMode` and the `-m/--mode` CLI flag in `read_args.rs`.
- Headless mode. ~~First removed together with the crutches (the window was always created by `run`; `shared.gpu`/`shared.render` only meant "not created yet").~~ [rejected] The author stopped this mid-work: **headless stays**. Second variant, also replaced mid-work: ~~`Engine::new()` without arguments + a separate `engine.run_headless(initial_world)`; the sound device opened later by `audio.start_sound_engine()` in windowed `run` only.~~ The author then said the `headless` argument can stay in `Engine::new` and be set from the command line (or by a separate entry point). [accepted] Final:
  - `Engine::new(headless: bool)` → `EngineSharedSystems::new(headless)` → `AudioSystem::new(headless)` (no sound device, as before the round); `Engine.headless` field.
  - `engine.run(initial_world, graphic_backend)` is the single entry: if headless → private `run_headless` (no winit, no window; `Engine::start(None, ..)` loads the initial world without GPU; a plain loop: `tick()`, sleep until the next target tick, until `quit_requested`); otherwise the winit loop, where `Init` calls `Engine::start(Some(window), ..)`.
  - CLI flag `--headless` (`Args.headless`); `game_client_main.rs`: `Engine::new(args.headless).run(start_level, args.graphic_backend)`.
  - `shared.gpu` / `shared.render`: `None` before the window is created and always in headless mode.
  - Verified: `game-client --headless` loads `capture_the_flag_map` (descriptor without GPU, physics init, root world set) and ticks without panics (stopped by `timeout` after 8 s: nothing in that world requests quit); the windowed run is unaffected.
- Found while testing headless (pre-existing, not related to this round, not fixed): `obstacle_course_levels/*` and many other maps lack the required `it_is_2d_3d_example` property, so `WorldDescriptor::load` panics on them; the old `level.rs` had the same `expect`. Recorded for §24.10.
- The two remaining alternative entry points (`*_without_autoupdate`, `*_with_windows_command_line`) are left as they are: they use a much older API (`InputMaster`, `players_settings`, a world-setup closure) and were not compiling before this refactoring either; `AGENTS.md` says so.
- Removed `DisconnectReason::ClosedLocally` and `Transport::Closed` were only created by the "disabled" net branch.
- Check: `cargo build --release --bin game-client` OK (702 warnings, same as before the round), `cargo test` OK (0 tests), client run: `engine: init` → window/GPU → `capture_the_flag_map` loaded, exit code 0, render thread stopped cleanly; `--headless` run as above.

**Round 7, point 5 (2026-11-13): alternative entry points, leftover files, warnings.** The author asked: update the alternative entry points to the new API and add them to `Cargo.toml`; delete files that are no longer needed; fix all warnings (unused imports/variables etc.), leaving alone a warning that can't really be fixed.
- **Entry points.** `game_client_main_without_autoupdate.rs` (no WinSparkle) and `game_client_main_with_windows_command_line.rs` (no `windows_subsystem = "windows"`, i.e. a console window on Windows; calls `winsparkle::init()`, a no-op stub on other platforms) rewritten to the same 4 lines as `game_client_main.rs`; their old bodies used a long-gone API (`InputMaster`, `players_settings`, a world-setup closure). `[[bin]]` entries `game-client-with-windows-command-line` and `game-client-without-autoupdate` enabled; CI still builds only `--bin game-client`. The other alternative entry points had been deleted by the author; their commented-out `[[bin]]` entries (and the old `game_client_main_web` `[lib]` comment) were removed from `Cargo.toml`.
- **Deleted files:** `src/assets/mod.rs` (one commented line, not declared), `src/actor/projectiles.rs` (fully commented out, not declared), `pkg/` (output of the removed wasm build). Kept (the author's decision): the unused shaders `raymarch_shader_with_bsp_tree.wgsl`, `raymarch_shader_for_2d_3d_example.wgsl`, `raymarch_shader_for_2d_3d_example_with_bsp_tree.wgsl` (generator output / the 2D-3D example, relevant for the planned tutorial screen); the `raymarch_shader_generator` bin source `src/raymarch_shader_generator.rs` was deleted by the author earlier (the generator module itself is used at runtime by `WorldDescriptor::load`).
- **Manifest:** `resolver = "2"` sat inside the last `[[bin]]` table and was ignored ("unused manifest key") → moved into `workspace = { …, resolver = "2" }` (no behaviour change: edition 2021 implies it).
- **Warnings: client 702 → 2, servers 33 → 0** (debug and release, all bins). How:
  - `cargo fix` (machine-applicable: unused imports, needless `mut`, elided lifetimes `'_` in signatures, …).
  - `unused_assignments` (render data): `let mut x = 0u32;` initializers that were always overwritten before being read → `let x: u32;`; counters `x += 1` whose value was always overwritten later → removed; computed-but-never-read amounts (`inf_cubes_amount` etc., their use is commented out) → `_` prefix.
  - Note: Found in `static_render_data.rs` (pre-existing, same in HEAD): `unbreakable_spheres_amount` is assigned twice — the second time with the s-spheres count (`- unbreakable_s_spheres_start`) — and `unbreakable_s_spheres_amount` is never set (stays 0); also the "unbreakable" sections are filled from `s_cubes`/`s_spheres`, not from the `unbreakable_*` vectors. Looks like copy-paste bugs, but fixing them changes rendering. Only the dead first assignment was removed (identical behaviour) and a `NOTE` left; [open] for the author.
  - `deprecated`: rand 0.9 renames (`thread_rng` → `rand::rng`, `gen_range` → `random_range`, `gen_bool` → `random_bool`); `fyrox_core::rand` (rand 0.8) calls left as they are (not deprecated there).
  - `unreachable_patterns`: `_ => {}` arms after an exhaustive match of a one-variant enum removed (4).
  - `private_interfaces`: `raymarch_shader_generator::Slice` made `pub` (it's a `pub` field type of `SliceInfo`).
  - `dead_code` (118): real leftovers deleted (unused constants, three copies of `get_random_vec4`, `set_transform` helpers, `FrameCollidersBuffers` (unused since the physics migration), `StaticColliderData`, `Spawn`, unused enums and getters, `MouseAxis`, `ButtonActionsLinkedKeys`, …); fields that were only written at construction deleted (`ui_sampler` — the bind group keeps the sampler alive —, `need_to_redraw`, `PlayersDollCollider::{friction, bounce_rate}`, five second-jump/base-effect fields of `PlayerInnerState`, …; `DropedRotatorTool` fields got a `_` prefix). Intentionally kept but unused code is marked `#[allow(dead_code)] // <reason>`: the nested-world API (`World::set_render_rate`, `send_*_message`, `take_outer_world_messages`, `OuterWorldMessage` variants, …), sound/UI/time/input APIs, and game content not wired into the current maps/modes (observer, 2D-3D example player, ObstaclesGun (module-level), MoverW, MoveWBonus, W floors/roofs, …). The `actor_wrapper!` macro now accepts attributes before a line (they go to that `ActorWrapper` variant).
  - Debug-only: the `DEBUG_*` statics in `kinematic_collider.rs` are used only under `#[cfg(debug_assertions)]` → declared under the same `cfg`; `log::warn!` reads them by value (`{ X }`) instead of creating a shared reference to a `static mut`.
  - Servers: `game_server` now uses its own lib crate `client_server_protocol` instead of `mod client_server_protocol;` (the module was compiled twice, and the protocol functions only the client uses warned as dead code); `NetCommand` (an old client-side type, not part of the wire format) deleted from the protocol; unused imports/trait/enum removed; config fields that are read but not used marked with `allow`.
  - `cargo test --workspace` failed before this round too: the `game_server` test binary panicked in the `blink-alloc` global allocator (allocator-api2 layout assertion; verified on a clean HEAD worktree). The global allocator is now `#[cfg(not(test))]` in `game_server` (no change for normal/release builds). Now all tests pass.
  - **Left (can't be fixed without a decision):**
    1. `Cargo.toml`: "only one of `license` or `license-file` is necessary" — licensing is in flux, not touched.
    2. `players_doll.rs`: unreachable second `SpecificActorMessage::SessionControllerMessage` arm (pre-existing): the `TeamWin` wave effect of player dolls never runs, because the first arm catches all `SessionControllerMessage`s. Merging the arms enables the effect (a gameplay/visual change), deleting it loses the intended logic. [open] for the author.
    3. `game_client_main.rs`: `init_winsparkle` is never called (+ the non-Windows `winsparkle::init` stub unused in this bin). The call was removed from `main` in merge `54c1980`, so the shipped Windows client doesn't start WinSparkle auto-update checks at all. Calling it would change what players get; deleting it would cement it. [open] for the author.
- Check: `cargo build --workspace` and `cargo build --release --workspace` — only the warnings above; `cargo test --workspace` OK; `game-client` windowed run (map loaded, exit 0, render thread stopped), `game-client --headless` and `game-client-without-autoupdate --headless` (map loaded, ticking).

**Round 8 (2026-11-13): the author's answers to round 7 point 5, licenses, working patterns.**
- **blink-alloc / test panic.** The author suspected the server test panic started after a `cargo update` and asked to try a targeted update of blink-alloc (or roll back to the version of a year ago). Findings:
  - `game_server` has no own `Cargo.lock` (workspace member → shared root lock). The lock had **blink-alloc 0.3.1 / allocator-api2 0.2.21 in every commit since 2025** (checked `16b20a0` … `f3e2128`), so the panic did not come from an update. ~~`cargo update` caused it~~ [rejected].
  - `cargo update -p blink-alloc` → 0.4.0 (+ allocator-api2 0.4.0): the panic stays. Lock rolled back to 0.3.1.
  - Root cause (read in the crate source, both 0.3.1 and 0.4.0, `src/global/sync.rs`, `State::resize`): in direct mode (`enabled == false`, the default) the condition is inverted — `if old_layout.size() >= new_layout.size() { grow(..) } else { shrink(..) }`. Debug builds hit allocator-api2's `debug_assert!` in `shrink`; release builds skip the assertion but then pass a false `assume(new_size <= old_size)` to the optimizer (undefined behaviour in principle; in practice `realloc` still gets the right size).
  - The game never calls `GlobalBlinkAlloc::blink_mode()`, so the allocator always runs in direct mode = the system allocator plus this bug: **it gives no speed-up at all**. This also explains `d3cda7e` ("disabled global allocator in debug build").
  - [open] Decision for the author. [proposed] Claude's recommendation: remove blink-alloc from client and server (the system allocator is what actually runs now; the "no allocations" principle is achieved in code, not by the allocator). Real blink mode would need an arena reset scheme (`reset()` at a point where nothing allocated in blink mode is alive) — not suitable for long-lived game state without a redesign. Until decided: `game_server`'s global allocator stays `#[cfg(not(test))]` (comment explains the bug).
- **`resolver = "2"`:** kept in the `workspace` table (the author once set it for wgpu; with edition 2021 it's the default anyway, so it's harmless).
- **Shaders:** `raymarch_shader_with_bsp_tree.wgsl` and `raymarch_shader_for_2d_3d_example_with_bsp_tree.wgsl` deleted (the BSP shader is generated at runtime now); `raymarch_shader_for_2d_3d_example.wgsl` kept (temporarily unused).
- **Player dolls' team-win effect fixed:** the `TeamWin` arm moved into the first `SpecificActorMessage::SessionControllerMessage` arm of `PlayerDoll::recieve_message`; the unreachable duplicate arm deleted. `TeamWin` is a broadcast (`RemoteBoardCastMessage` → `send_broadcast_message`), so the dolls of the winning team now spawn their wave.
- **WinSparkle auto-update restored in `game-client`:** `#[cfg(target_os = "windows")] winsparkle::init();` directly in `main` (right after the logger, where it was before `54c1980`); the helper `init_winsparkle()` and the non-Windows stub `winsparkle::init() {}` deleted; same inline `cfg` in `game-client-with-windows-command-line`. Checked with `cargo check --target x86_64-pc-windows-gnu --bins`.
- **Copy-paste bug in `StaticRenderData` fixed:** the undestroyable GPU arrays are packed from `unbreakable_cubes/spheres/sph_cubes` and `unbreakable_s_*` (were: the destroyable `cubes`, `spheres`, `inf_w_cubes`, `sph_cubes` and their sticky variants); `unbreakable_spheres_amount` / `unbreakable_s_spheres_amount` are set correctly (the s-count used to overwrite the first, the second stayed 0); no infinite-W cubes in the unbreakable sections (there are no unbreakable ones). This now matches `DynamicRenderData`, which already used `sd.unbreakable_*`. The BSP generator reads `unbreakable_*` directly, so the CTF map looks the same (screenshots before/after compared); maps on the plain `raymarch_shader.wgsl` now get their unbreakable objects in the buffer.
- **Licenses** restored to the September 2025 release (`a9f478f`, "Add AGPLv3 license and comprehensive attribution file" + README/CREDITS of that time), after the temporary switch to MIT:
  - `LICENSE.txt`: AGPL-3.0 text (was MIT); `Cargo.toml`: only `license = "AGPL-3.0-or-later"` (the release had `license = "AGPL-3.0"` + `license-file`; cargo warns that only one is needed — an SPDX id is enough, and "-or-later" matches the "version 3 … or any later version" wording); README and CREDITS.md already matched the release (code AGPL-3.0, original assets CC BY-SA 4.0, `media/` All Rights Reserved, third-party sounds CC0/CC BY 3.0/4.0, WinSparkle, matchbox NOTICE).
  - File headers: ~~the release's full AGPL notice restored in every file~~ [rejected] the author stopped it mid-work: the license is defined by the license file, **files carry only the two copyright lines**. Done so everywhere (also the 4 render files that had no header, `Cargo.toml`, `deploy.yml`, `installer.iss`).
- **`AGENTS.md`:** new section "How to approach a task (lessons from working with the author)" — finish tasks completely (move modules, docs, issue, all bins), freedom to remove crutches but distinguish crutch vs capability (headless), fix obvious bugs found on the way and verify, root cause over masking, answer "is it better" with an opinion and act, no trivial `cfg` wrappers, zero warnings, the author works on the same branch in parallel, how to verify (incl. the Windows GNU check), Russian + explanations, targeted `cargo update -p` only. Licensing and header conventions rewritten; blink-alloc note.
- Check: `cargo build --workspace` and `--release --workspace`: **0 warnings**; `cargo test --workspace` OK; `cargo check --target x86_64-pc-windows-gnu --bins` OK; client windowed (CTF map, exit 0) and `--headless` OK.

**Round 9 (2026-11-13): wrap-up before the author's review.** The author: allocators are the next discussion stage; now he reviews the refactoring manually and commits if accepted. Last requests, done:
- **Issues hierarchy:** issues are for the whole project, one folder per area (`programming`, later `game-design`, `level-design`, `art`, …). All current issues moved to `issues/programming/`; all references updated (`AGENTS.md`, issues, code comments).
- **Done naming:** a finished issue is renamed with a `done-` prefix before the date (`done-YYYY-MM-DD-name.md`), so open and done tasks sort apart. This issue is **not** renamed yet (awaiting the author's acceptance); its header now has a short "current state" summary so it works as context.
- **`AGENTS.md`:** README must always be updated together with code changes (new "README" section; also in "Finish the job completely"); issues section rewritten (areas, naming, what an issue contains).
- **README:** the "written manually" statement (it was in `71c5ff2` and got lost in the October branch merges) replaced by a dated one: fully manual up to and including `f3e2128` (2026-11-03), agent-assisted from the engine refactoring on (the first commit after `f3e2128`); the author's name removed from that statement (it stays in the legal license lines). New section on agent-assisted development (`AGENTS.md`, `issues/<area>/`, what an issue contains, open/done naming). Updated for the refactoring: settings files (`connection_settings.json` next to the executable, `user_settings.json` in the config dir; the old `settings.json` is gone), CLI flags (`-b`, `-l`, `--headless`, `-v`), client bins.
- **`.gitignore`:** `CLAUDE.md` ignored (it only imports `AGENTS.md`; the repository keeps `AGENTS.md`).
- **New issue** `issues/programming/global-allocator/`: the blink-alloc findings (round 8), the author's answer, the recommendation and the open questions.

**Round 10 (2026-11-13): tick rate, English in the engine.**
1. The engine tick period isn't exactly 16.(6) ms on average (the deadline is `now + period` after wake-up, so wake-up lateness accumulates). Split out into its own issue: `issues/programming/engine-tick-rate/` (not part of this refactoring).
2. English in the engine (`src/engine.rs`, `src/engine/**`, incl. WGSL shaders) checked and corrected:
   - comments (typos and grammar; commented-out code left as is);
   - texts of `expect`/`panic!`/log messages: the ~300 map-parsing messages unified to `Wrong JSON map format: <what's wrong>` with correct grammar ("has no X property", "must be a boolean", ...), the other messages rewritten by hand;
   - identifiers with typos renamed everywhere they're used (also in actors and shaders): `bounce_coefficient`, `DEBUG_ITERATION_COUNTER`, `Sound::FlagCaptured`, `Sound::PlayerHit`, `NewProjection`, `UIElementType::*ScannerDisplay*`, `EnvironmentVisualSettings`, `fresnel_color`/`fresnel`, `hit_pos`, `hit_actors_id`/`hit_actors_team`, `w_scanner_ring_intensity`, `spherical_areas_metadata`, `split_screen_in_2d_3d_example`, `rotator_tool_equipped`, `move_left`, `movement_mult`, `absorbed_translation`, `probable_translation_dir`, `slice_info`, `swapped_actor`, `set/get_transparency`, `progress_bar_*`, `shader_uniforms_and_storage_bind_group_*`, `distance_from_collider_center`, `cast_sphere_*`, `kinematic_collider_intersects_area`/`is_intersecting`.
   - **Deliberately not renamed:** JSON map keys (`frenel_color`, `sticiness_negative_*`; renaming them changes the map format and every map file), asset file names (`player_hited.wav`), the intentional `recieve_message`, and game-logic / protocol names outside the engine (`DropedRotatorTool`, `FinalTrgger`, `trgger_orb`, `YouHitedMe`, `FlagStatus::Droped`, `DropedFlag`, `hited_*` locals in actors). `undestroyable` (better English: "indestructible") is an established concept used in maps and shaders, kept.
   - Verified: release/debug workspace build with zero warnings, `cargo test --workspace`, Windows gnu check, the CTF map windowed (screenshot OK) and `--headless`; the obstacle-course and dithered hand-written shaders compile (checked with temporary map copies).

**Session wrap-up (2026-11-16).** The author ended the working session after round 10. Everything from rounds 1–10 is on `dev`, **uncommitted**, waiting for the author's manual review. Context was rechecked for handover: `AGENTS.md` (current issues, warnings, English and map-error conventions, known map problem); the map-loading problem found in rounds 7 and 10 got its own issue (`issues/programming/maps-out-of-sync-with-parser/`). Next possible steps, as the author decides:
- review/commit this refactoring, then rename this file to `done-…`;
- allocator discussion (`issues/programming/global-allocator/`);
- exact tick rate (`issues/programming/engine-tick-rate/`);
- the same English pass for actors and servers (offered, not requested yet);
- maps (`issues/programming/maps-out-of-sync-with-parser/`).

### 24.10 Still to verify manually (by the author)

Things the agent couldn't verify without playing:
- Gameplay: movement feel, jumping/W-jumps, all weapons, holes, scanner, flags, bonuses, respawn, death plane, HUD animations.
- Online with two clients: other players' dolls, shooting/holes sync, flags, score, team win, disconnect/reconnect UI.
- `i`/`O`, `P`, `J`/`K`, `N`/`M`, and that the values are restored after a restart.
- Window resize / fullscreen toggling (`Esc`/`Enter`).
- `-l obstacle_course_levels/level-1` (and `FinalTrigger` level transitions). Note: Found in round 7: these maps lack the required `it_is_2d_3d_example` property and panic in `WorldDescriptor::load` (pre-existing, same in the old `level.rs`). ~~`-m observer`~~ (removed, round 7 point 4).
- Performance on the author's gaming machine (the agent's comparison in §24.9 showed no regression).
- Round 8 fixes: the winning team's player dolls spawn a wave on team win (online); WinSparkle update check starts in the Windows `game-client`; unbreakable static objects on maps that use the plain `raymarch_shader.wgsl` (non-BSP) render correctly.


## 2026-11-16 — Review response and documentation migration

[accepted] The author requested concise task files plus separate context files, context chains with reasons for decisions, no dates in open filenames, completion dates only after acceptance, and visible high priority. Implemented using stable task directories, a paired `-context.md`, `high-priority-` on both high-priority filenames, and `done-YYYY-MM-DD-` on both at completion (filename prefixes [superseded] on 2026-11-20 by separate open/closed task lists; see the [issue README](../../README.md#tracking-convention-history)). The stable-directory mechanism is the implementation choice for avoiding broken cross-task links; it does not require symlinks or extra redirect files. See [AGENTS.md](../../../AGENTS.md#issues-and-decision-context).

[accepted] The original seven issue texts remain in their context files; earlier section numbers and rejected alternatives are preserved. Main files now summarize current scope, decisions and open choices. The former AGENTS architecture reference moved to [docs/engine-architecture.md](../../../docs/engine-architecture.md), with known limitations identified. README and source-comment references were updated. No gameplay code was changed or issue accepted as complete.

[accepted] The author requested 12 follow-up tasks; [the index](../../README.md) maps them to review points. High priority is 2 (connection lifetime), 3 (message order), 6 (main actor ID collision), 10 (async preparation). GPU grouping and overflow protection are long-term. No standalone issues were requested for nested-world verification, performance instrumentation or a test suite; existing allocator/tick-rate issues remain.

[rejected] Superseded premise, not yet changed code: message processing cannot generally treat state assignments as order-independent. FIFO implementation remains to be designed in [message ordering](../message-ordering/); previous LIFO reasoning is retained above.

[proposed] Reopened resource decision: the author prefers investigating a special non-cloneable prepared world resource over mandatory actor cloning, to move resource work out of activation. [Async world preparation](../async-world-preparation/) records both options, earlier side-effect constraints and the distinction between cloning handles and actors. This is not approval to implement either design yet.

[accepted] AGENTS rules now allow justified ownership/invariant wrappers and distinguish allocation avoidance from measured correctness/performance. Logical responsibility, not file length alone, drives module splitting. Existing licensing, release safeguards, headless support and the author's control over large architecture changes are preserved.


## 2026-11-16 — Final documentation and license consistency check before dev commit

The author requested one last README/architecture/license check before committing and pushing to dev. Corrected README output paths, the server configuration version example (0.5.10, with an instruction to use the client product version), map limitations, actual online hotkey and bounded connection retries. Updated the architecture reference for settings lookup, clamped world time, remote sender IDs, render-slot cleanup and the distinction between gameplay conventions and engine restrictions. Replaced stale “core principle 5” references after AGENTS restructuring.

License policy is unchanged: original code AGPL-3.0-or-later, original assets CC BY-SA 4.0, media reserved, third-party licenses retained. README/CREDITS now explicitly say “version 3 or later” and exclude third-party code from the original-code statement. Both server manifests now declare the existing AGPL-3.0-or-later license and carry the project's copyright header. LICENSE.txt matches commit 9b0e6d3 byte-for-byte; the [GNU text](https://www.gnu.org/licenses/agpl-3.0.txt) was consulted as the canonical reference.

Corrected the existing `shifting_along_w.wav` attribution from CC BY 1.0 to CC0 1.0: the [source page](https://freesound.org/people/MrJmix/sounds/331078/) identifies CC0 and links to the [CC0 1.0 deed](https://creativecommons.org/publicdomain/zero/1.0/). This corrects catalog metadata, not the author's licensing choice. All credited local sound paths exist; four sounds not listed as third-party remain covered by the existing original-assets declaration (their authorship was not independently established in this check). Added explicit CC license links.

The installer previously displayed the root license but did not explicitly install the attribution/catalog and bundled third-party notice files. Added copies of LICENSE.txt, CREDITS.md and WinSparkle/Matchbox licenses/NOTICE. This is packaging only; no deployment or version bump. Full transitive-dependency license/provenance auditing is outside this consistency check.

Verification on the uncommitted dev tree: `cargo check --workspace --bins --locked` passed without compiler warnings; `cargo metadata --no-deps --format-version 1 --locked` confirmed all four package license declarations and unchanged product version. Local Markdown links and all non-generated installer source paths exist. Removed one duplicate sound attribution row without removing any unique source credit. Windows/Inno Setup installer compilation was not run on this Linux environment; the packaging changes were checked statically. No commit, push or production connection was performed.
