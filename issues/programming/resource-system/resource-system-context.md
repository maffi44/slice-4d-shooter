# Context: Resource System

[Current task](resource-system.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

## Preserved history (before the 2026-11-16 documentation migration)

The following original issue is preserved in full. Its old filename rules, status summaries and “final” architecture headings describe their original rounds and are superseded by the current task and AGENTS.md where they disagree. Section numbers remain available for older references.

# Resource System

- **Created:** 2026-11-12
- **Status:** 🔵 Backlog, high-level. Design comes after (or alongside) the engine refactoring.
- **Related:** `issues/programming/engine-refactoring-world-isolation/` (split out of it; see §11–§12 there)

This issue is intentionally **high-level**. It doesn't fix implementation details or how the system integrates into the new architecture, because that architecture is still being finalized.

## Motivation

Audio and UI currently preload and hard-code their resources inside the systems themselves (sounds via `SoundBufferResource::new_generic(include_bytes!(..))` in `audio.rs`, textures via `include_bytes!` in `ui.rs`, etc.). Levels have their own ad-hoc loading with path fallbacks plus embedded maps in `level.rs`. The author wants actors to request the resources they need, similar to Fyrox and other engines.

## Requirements (from the author)

- **A shared resource-loading system**, part of the Engine Shared Systems.
- **Typed requests by path**, e.g. `load_resource(path)`, with a known type: audio, texture, level/world descriptor, others.
- **Sync loading:** load during the call; the caller blocks until done.
- **Async loading:** start a task and return a **handle** to check status and get the result later. Undecided whether this is a tokio `JoinHandle` or the system's own handle. (Claude's earlier note: an own cloneable handle fits caching better, because several actors may wait on the same in-flight resource, and a `JoinHandle` is neither `Clone` nor reusable.)
- **Usage pattern:**
  1. the actor requests a resource
  2. the Resource System provides it
  3. the relevant shared system (Audio, UI, …) creates an element from it
  4. the actor keeps the returned element handle (sound handle, UI element handle)

  For a level/world descriptor the actor just keeps the resource and passes it to the `World` constructor.
- **Caching for the whole app session** (launch to exit; not persisted across runs):
  1. request
  2. check the cache
  3. return the cached instance if present
  4. otherwise load (sync or async) and store it
- **Reuse:** many actors of the same type share already-loaded textures, sounds, etc.
- **Preloading**, e.g. load the first level's resources while in the main menu. The menu will itself be a `World`, so its actors may just start async loads, and a dedicated global preload mechanism may be unnecessary.
- **Convenient declaration in actor code**, possibly with embedded resources via `include_bytes!` wrapped in a macro (working title `include_bytes_and_name!`) that yields both the bytes and a **unique key**:
  - The author prefers the **source path (possibly plus resource type)** as the key over a hash (collision concern).
  - This must be reconciled with path-based `load_resource(path)`.
- **Async `World` creation** (shader compilation, pipeline creation) must be supported. See the refactoring issue §12: the current direction is caching an immutable `WorldDescriptor` (incl. pipelines), with `World::new` cheap and holding all creation events.
- **Heavy work must not disturb the render loop.** Where it runs (tokio blocking pool, separate runtime, own pool, Fyrox `TaskPool`) is open; see the refactoring issue §12.5.

## Investigate: `fyrox-resource`

The author prefers a ready-made solution if it fits. It is already a dependency: `fyrox-resource 0.36.2`, and `fyrox-sound`'s `SoundBufferResource` is a `Resource<SoundBuffer>`, so audio already uses it.

What 0.36.2 offers (from reading the source):
- `ResourceManager::request::<T>(path) -> Resource<T>`
  - path-keyed registry, so caching is built in
  - `Resource<T>` is cloneable and `Arc`-based, with states Pending / Ok / LoadError, `is_loading()`, `is_ok()`, and it implements `Future`
- Loaders per file extension (`ResourceLoader::load(path, io)`), a pluggable `ResourceIo`, and manual `register(..)`.
- Built-in/embedded resources: `embedded_data_source!(path)` expands to `DataSource::new(path, include_bytes!(path))`, which is essentially the author's `include_bytes_and_name!` idea.
- Its own `TaskPool` (separate threads), independent of our tokio runtime.

Requirements on resource data types (`ResourceData` / `TypedResourceData`):
- `Visit + Reflect + Debug + Send + Downcast + Default + TypeUuidProvider`
- plus `save()` / `can_be_saved()`

What these mean:
- **`Visit`:** Fyrox's serialization (save/load of scenes and resource state). For runtime-only data it can be a no-op, or derived with `#[visit(skip)]` on fields (skipped fields need `Default`).
- **`Reflect`:** runtime reflection (field enumeration, used by the editor/inspector). The derive requires every field to be `Reflect` unless marked `#[reflect(hidden)]`. wgpu types aren't `Reflect`, so they'd be hidden.
- **`Default`:** wgpu `RenderPipeline` has no `Default`, so it must be wrapped as `Option<…>`.
- **`TypeUuidProvider`:** a derive with a fixed UUID.
- `can_be_saved() → false` for runtime-only data.

Conclusion so far: this is **boilerplate, not a blocker**. Claude's earlier "awkward" overstated it.

Real points to evaluate:
1. **The loader model is path + extension.** GPU pipelines are not loaded from a file; they derive from level data + device + target format. They'd need either a loader that holds a wgpu `Device`, or manual `register` under a synthetic key.
2. **Every data access goes through a mutex** (`ResourceHeaderGuard` holds a `MutexGuard`). That's fine at load/creation time; don't access it in hot loops or on the render thread (clone cheap handles out once).
3. **Headless:** GPU-resource loaders need a device, so descriptors must work without pipelines.
4. **Version:** `Cargo.toml` has `fyrox-resource = "*"`. The lock pins 0.36.2, while the latest is 1.0.x, which reworked the registry (UUIDs/metadata). Adopting it means pinning a version deliberately and keeping it in sync with `fyrox-sound` and the future Fyrox UI.
5. **Thread pool:** its `TaskPool` would answer the "heavy work off the render path" question.

💡 Next step: a small spike that loads a sound, a texture and a level descriptor through `ResourceManager` (incl. one embedded resource), to decide fyrox-resource vs own implementation.

## Open questions

- `fyrox-resource` vs own implementation (after the spike).
- Handle type: own handle vs `JoinHandle` (vs `fyrox_resource::Resource<T>` if adopted).
- Key format and the embedded-resource macro; how embedded and file-system resources share one key space (cache → embedded → FS, like today's `load_level` fallbacks?).
- Sync request while the same resource is loading async: block, or return the pending handle?
- Where heavy work runs.
- Whether a global preload API is needed, or actors in a menu `World` are enough.
- Migration plan for Audio and UI resources (UI likely together with the Fyrox UI rewrite).

## Discussion

1. **2026-11-12. Split out of the refactoring issue (round 3).** The author described the concept (typed path requests, sync/async, session cache, reuse, preloading, embedded resources with a path key, async world creation) and prefers `fyrox-resource` if `Reflect`/`Visit` aren't a blocker. Claude read the 0.36.2 source: boilerplate, not a blocker; real points are the loader model, mutex access, headless, version pinning, `TaskPool`.
2. **2026-11-12. Round 4 of the refactoring discussion.** `WorldDescriptor` was approved as the cached, immutable, preloadable unit for worlds (shaders + pipelines + parsed map + actor descriptions). Until this system exists, the refactoring loads descriptors with `WorldDescriptor::load` on `shared.runtime.spawn_blocking` and `JoinHandle`s (the same pattern `Level::load_level` uses today). That's the integration point for this issue later.
3. **2026-11-12. Round 5 of the refactoring discussion: target usage flow (author).** Example with `FinalTrigger`:
   1. On creation the actor requests the next world's `WorldDescriptor` from the Resource System.
   2. If it was loaded before, the Resource System returns it; otherwise it starts an async load and the actor gets a handle.
   3. When it's time to change levels, the actor sends `OuterWorldMessage::ReplaceRootWorld(..)` carrying the prepared descriptor (`Arc`/`Box`, so the enum stays small).
   4. No engine-level `PreloadNewLevelAsync` / `LoadNewLevel*` commands; those concepts become redundant.

   Also decided in round 5: a `World` does **not** keep a reference to its descriptor. An actor that wants to recreate a world keeps the descriptor resource (handle) itself. So the Resource System's cache (or the actors holding handles) is what keeps descriptors alive.

## 2026-11-19 — approved integration

✅ The author approved Fyrox with a project-owned retention policy and reusable actor descriptors. A held untyped Fyrox handle pins a session resource. Retention can be promoted after its first request. Full world instances and cloned actors are superseded by factories capturing validated constructor arguments and prepared sound handles.

The sound collection stores Fyrox's shareable untyped handles. Its constructor requests only `SoundBuffer` and checks the type on playback. Fyrox 0.36.2's typed sound handle carries the thread-safety bounds of streaming decoder data, so sharing it directly inside an immutable map descriptor fails Rust's `Sync` bound. Gameplay APIs retain typed access.

The existing two-phase startup continues to overlap map parsing with device creation. Later world requests submit work to the existing Tokio runtime through a Fyrox loader. The loader holds a weak reference back to the resource system. Built-in sound decoding runs on Fyrox's task pool. Source creation and world callbacks run during gameplay.

✅ Start next-level preload from `on_added_to_world` when enabled by the map. A trigger descriptor stores the destination key. A map can target itself without waiting on or retaining itself. The separate `async_load` property now reads its own JSON key; the previous parser read `async_preload` twice.

### Verification on Linux, working tree based on `76d73d9`

- Release workspace build, Linux workspace bins check, Windows GNU bins check and workspace tests passed with the lockfile. The Matchbox integration tests used local sockets. The two new resource tests run in each of the three client binaries.
- The retention test promotes an existing sound to session lifetime, drops caller handles, advances cache maintenance by 61 seconds and verifies resource identity. The world test requests one map twice, constructs separate actors, changes one actor's ID and verifies failure delivery for a missing map.
- The no-updater release client reached `engine: ready` headless and in a Vulkan window on an RTX 3070 Laptop GPU. Graphical CTF startup loaded 25 gameplay WAVs. Headless used zero sound loads. All smoke runs used a loopback matchmaking URL and isolated settings. Audible playback of each sound was not manually checked.
- Temporary maps outside the repository used the current schema, an obstacle-course player, a trigger orb and a final trigger. Headless gameplay exercised repeated self-transitions and an A → B → A round trip. The latter logged one background B load and reused the initial descriptor on return.
- The original obstacle-course level 6 still fails initial parsing on its missing `it_is_2d_3d_example` field. Legacy data migration belongs to [map compatibility](../maps-out-of-sync-with-parser/).

## 2026-11-20 — author review and revised implementation

❌ The November 19 sound enum, per-actor sound groups and actor descriptor factories required asset declarations in several places. The author wants each owner to declare a typed field and request its file by path.

✅ Each actor/device constructor now requests its own handles with a scoped resource context. Embedded assets get a key and static bytes from one literal. The initial preparation waits for exactly those requests. Gameplay-time requests can start a new background load. An audio source requested during decoding can already receive playback controls. Headless actors omit the audio handle.

✅ The shared world descriptor retains parsed JSON and a ready actor set. On each call it constructs the next set synchronously, swaps it with the spare and releases the lock before world registration and callbacks. The caller gets an active world directly. Strong handles to the destination world of a final trigger are acquired only after activation. Synchronous refill trades one spare actor set and constructor work per activation for simple, repeatable ownership.

The existing gameplay device slots already require `Box<dyn Device + Send>`. Compilation confirmed that the new actor sets can cross the worker boundary. Enum dispatch remains an independent design choice.

Verification used an uncommitted Linux working tree based on `76d73d9`. Workspace release build, workspace tests with local Matchbox sockets, Linux and Windows GNU bins checks passed. The Vulkan release client reached ready on the default CTF map with 22 distinct embedded WAV loads. Local headless fixtures exercised A → B → A and self-transitions. Each resource test runs in all three clients. The retention test uses the same embedded request API as actors. Individual audio playback and the worst-case synchronous refill time were not measured.

## 2026-11-20 — caller contract and nonblocking transitions

✅ Agree on the caller’s model before selecting internal mechanisms. The author wants each stage to cover its current use cases cohesively while keeping cognitive and runtime costs low. Apply this to new systems and revisions, recording intent in the architecture reference and decisions in the task.

✅ Actors request a handle, inspect it and consume ready data. A single nonblocking inspection distinguishes pending, success and failure. The final trigger now polls the destination and releases the resource guard before sending its message. This supersedes the blocking branch retained on November 19. Both legacy boolean values are accepted, and the field can be omitted. Audio continues to accept pending buffers.

Verified on the uncommitted Linux tree based on `76d73d9`. Release workspace build, workspace tests and Windows GNU bins check passed. The handle test covers pending work, contention on a ready resource, success and failure. With loopback connection settings, headless fixtures completed A → B → A with preload disabled, using legacy false on A and an omitted flag on B. A missing destination produced one trigger error and the current world kept running until the smoke-test timeout. The Vulkan client reached ready on the default CTF map.
