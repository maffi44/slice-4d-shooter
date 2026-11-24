# Unified resource requests — discussion and design context

[Current task](unified-resource-requests.md)

## 2026-11-23 — what callers should write

[accepted] The author wants to design from the call site. An actor requesting a shot sound knows its key and required lifetime. A trigger knows the destination name and when to switch. The engine loader knows whether the startup world should stay resident. They should all be able to express these choices directly. The signatures below are sketches; the examples use a prospective menu.

```rust
// Actor field supplies the expected sound type.
shotgun_shot_sound: resources.request(embedded_asset!("sounds/shotgun_shot.wav"), Retention::Session),

// EngineLoader starts the request early and awaits it after publishing GPU readiness.
let menu = resources.request::<WorldResource>(menu_key, Retention::Session);
let ready_menu = menu.await?;

// A trigger keeps this handle between ticks and uses try_read when it needs the result.
let level = resources.request::<WorldResource>(destination_key, Retention::WhileUsed);
```

The existing [resource system](../resource-system/) already uses a Fyrox typed handle with an immediate return, asynchronous completion and nonblocking tick inspection. A session request retains an additional handle until shutdown, including when promoting an existing request. Ordinary resources expire after 60 seconds of unused cache-update time. The present differences arise before or around that common request.

## What has already been solved

[accepted] The [embedded asset catalog](../embedded-asset-catalog/) removed the runtime path-to-bytes registration and its mutex. Separate invocations of `include_bytes!` had produced duplicate WAVs in the release client. Direct literal markers now select what ships; the generated catalog has one inclusion per key. It provides static source bytes. Fyrox separately caches the prepared resource. A map name read from CLI or JSON cannot itself declare a bundled file at build time. If bundled maps move into this catalog, their inclusion still needs an explicit declaration.

## Choosing bytes and preparing the typed resource

[proposed] On a cache miss, resolve the logical key, borrow bytes from the catalog or read an external file asynchronously, then pass the bytes to the responsible type loader. Keep borrowed embedded bytes and owned file buffers valid for the operation; allocating a common intermediate copy is optional. WAV decoding, JSON parsing and GPU pipeline preparation have distinct costs. The contract should make a future image decoder easy to register without bringing dynamic textures into scope. See [render resources](../render-resources/).

[open] The author's first preference is embedded lookup followed by disk on a miss. Today's map lookup checks several external JSON locations first and then a built-in table. This allows local overrides of shipped maps. Decide what an ordinary key and an explicit disk path mean when both sources exist; consider a path-specific request only if needed. Define cache identity for startup, transitions and CLI names so the same resolved map shares a descriptor and deliberately distinct files remain distinguishable. Fyrox currently selects loaders by extension and checks the requested type UUID; static type inference alone does not tell it how a bare level name maps to a loader. Today `request_world` appends a synthetic `.world` and fixes retention at `WhileUsed`.

[proposed] Publish completion directly to the pending handle. A disk read can feed the decoder on completion. Waiting for a resource-system tick to forward bytes would delay callers and could stall startup, which awaits resources before simulation begins.

## Headless and preparation dependencies

Today `ResourceRequests::sound` skips the entire request in headless mode and returns `None`. A world is still required there. During [world preparation](../async-world-preparation/), the scoped request context records handles actually requested by actor constructors. Two worlds can request one shared WAV; each preparation must observe its readiness and failures even if the decoder runs only once. A self-transition is different: the trigger requests its destination on activation or transition, outside the dependencies needed to finish preparing that same world.

[open] Keep these semantics while designing the common entry point. Explain how the requested type affects headless availability and what Rust can infer from an optional sound field. Moving all policy into the decoder would start a sound request before headless can skip it and would miss dependency collection on cache hits.

## Startup, retention and ownership

Today `EngineLoader` creates the resource system early, starts `WorldPreparation::load` directly on Tokio, overlaps parsing with GPU setup, finishes the descriptor, registers the world loader, then inserts the initial descriptor in the manager. That insertion does not pin the resource for the session. Later transitions go through the manager. See [nonblocking startup](../nonblocking-engine-startup/) for the window thread, cancellation and runtime ownership constraints.

[proposed] Register world loading early enough for the first request to use it. CPU preparation can begin immediately; only the graphics-dependent finish needs the device. The loader must be able to finish headless or resolve its wait on initialization failure or cancellation. Preserve event-loop ownership of the window and keep blocking driver work off async workers.

[open] A future menu is a plausible session resource; today's default startup map is CTF and `-l` can select another. Specify retention at the call site that knows the intent. Pinning a menu keeps its descriptor, geometry, pipelines and spare actor set available. Returning creates a fresh simulation from the descriptor. UI state that should survive a game has a separate owner. The contract should support this example without requiring an actual menu implementation.

## 2026-11-23 — proposed contract for review

[superseded] The initial proposal put one public `request<H>(key, retention) -> H` on the existing request context. The concrete result type selected both the resource data and the headless policy. The author prefers the explicit media call below so that choosing optional behavior is visible to the caller. The common cache, retention and dependency handling still apply.

[proposed] Use a normalized logical asset key as the cache identity within each type. A level name from the CLI or a trigger normalizes to the same maps key as its bundled declaration, preserving the subdirectory. Fyrox 0.36.2 indexes by path and selects loaders by extension. Map the typed key to a private manager path with an unambiguous loader suffix before requesting it. Keep physical candidate paths private to the byte resolver. A second caller can acquire the pending entry before any filesystem lookup finishes. Distinct logical keys can intentionally denote separate files with the same basename. Physical aliases such as symlinks are outside this identity contract.

[proposed, open for author decision] Preserve external overrides for logical map names, including the current ordered candidate locations. Keep catalog-first lookup for sounds. Only a missing candidate falls through; report an unreadable or malformed override with its path. This preserves editing a bundled map and launching it by its usual name. Applying catalog-first to maps would instead require selecting the override explicitly. Keep direct literal bundled-map declarations in the generated catalog when replacing the existing include table. Pass borrowed catalog bytes or an owned file buffer to the same preparation function per type. A future file texture adds its own type policy and preparation (decode, then device upload) through this contract.

[proposed] At startup register the loaders before requesting the initial map. Pass the world loader a clonable receiver for device readiness (e.g. Tokio watch). The engine loading worker is the sole publisher. Headless is terminal from creation. Publish the device as soon as it is available, before awaiting the world. CPU preparation and dependency requests can run immediately. Only graphical finalization awaits the device. A failed or cancelled startup must wake waiters even on early return or panic: a dropped publisher while still pending is terminal. Keep the last successful device available for subsequent loads. Do blocking CPU and driver work on loading workers, and preserve event-loop surface ownership and shutdown coordination for already-started work. Publish the descriptor or error directly through the Fyrox handle; no simulation tick is needed. Use WhileUsed for today's CTF/CLI entry and Session when a future caller explicitly chooses a resident menu.

## 2026-11-23 — explicit media requests and map overrides

[proposed] The author suggests `load<T>(key, retention) -> Resource<T>` and `load_media<T>(key, retention) -> Option<Resource<T>>` for sounds and textures. Both names remain under discussion. The media call checks headless at entry and immediately returns None there, before cache lookup, pinning, dependency collection or I/O. Otherwise it wraps the ordinary load in Some. The same graphical request through either method shares the handle. The field type can infer T in both calls.

[proposed] A required GPU-only resource in headless receives a handle with a terminal load error. Perpetual Pending would hang startup or a dependent world, and a trigger would retry every tick. A graphical device still initializing can genuinely be pending. A headless world is supported and completes CPU preparation. Sound-buffer decoding itself is CPU work and can be requested explicitly if a caller needs samples. Gameplay sounds use the media method.

[open] The author considered an external-first method used only at startup. With both a bundled and a disk map A, starting at A and returning through B would then depend on cache residency or require separate identities for two versions of A. The current recommendation is a documented per-type lookup order shared by startup and transitions, external-first for world names and catalog-first by default. Review the A → B → A example before adopting either choice.

## 2026-11-23 — approved implementation

[accepted] The author approved the explicit `load` and `load_media` pair, terminal errors for required GPU resources in headless, and disk-first map names at both startup and transitions. This supersedes the differing startup priority and return-type-directed dispatch proposals above. A missing local map tries the next candidate. A present unreadable or invalid override fails with its source path. The private manager key retains the directory and selects the registered loader by extension. Bundled map declarations moved from the parser to the generated catalog so disk and bundled inputs feed the same preparation.

[accepted] The shot sound is inferred from the actor field. The initial world is an ordinary `WhileUsed` request, and the trigger stores the same handle type outside constructor dependency collection. Fyrox polls a join handle for work on the existing Tokio runtime. File reads run asynchronously. WAV decoding, JSON/actor preparation and pipeline construction run on blocking workers. Fyrox's memory sound decoder requires an owned vector, whereas the JSON parser can borrow catalog bytes. Each preparation records handles on cache hits as well as misses, and completion wakes waiters directly.

[accepted] The engine loading worker owns the device publisher. A graphical world waits for the device only after CPU preparation. The last published device remains accessible for later worlds after startup. A closed publisher while pending yields an error. Window/surface creation still runs on the event-loop thread. Waiting for the initial descriptor also listens for cancellation, and runtime shutdown joins blocking work already in progress.

### Verification on Linux, uncommitted tree based on `75508b6`

- The release workspace build and Linux and Windows GNU bins checks passed without compiler warnings. All three client bins passed the resource, world, actor and settings tests. The full workspace test run hit socket permission errors in 24 Matchbox cases and the same nine client transport cases in each bin.
- A constructor's optional headless sound left the cache, retained set and dependency list empty. An explicit CPU sound request decoded. A required test GPU asset immediately reported an error. Two graphical preparation scopes observed the same failed WAV through one handle. Closing the device publisher caused a graphical world load to fail within the test timeout.
- In the world round-trip test, 61 seconds of simulated cache maintenance evicted an ordinary initial map. Its marked key reloaded a new descriptor from the same source. Pinning that map kept the descriptor across another game and return. The ordinary game expired after its caller released the handle. The same test passed in an isolated directory containing a local override of the shipped map. Independent actor sets and nonblocking trigger polling also passed.
- A copied no-updater release client ran with loopback connection settings and isolated preferences. From an empty directory the bundled CTF map reached ready. From the override directory it activated local A, followed its trigger to external-only B, and returned to local A. An external-only map selected by CLI completed a self-transition. No headless run loaded a WAV. A malformed local override failed startup with the exact file and JSON error.
- The Vulkan client attempted to start, but winit could not open the X display in the sandbox (`XOpenDisplayFailed`). Successful surface creation, rendering and close-during-device-initialization still require a graphical session.

## 2026-11-24 — review of actor access and preparation ownership

[accepted] The author wants actor code to call `shared.resources.load(...)` directly. It needs only `load` and `load_media`. Cache maintenance belongs to the engine and each world preparation owns its dependencies. Build-time discovery of map references has a [separate task](../discover-linked-maps/).

[proposed] Give shared systems a concrete `ResourceLoader` backed by the application resource system. Keep cache maintenance private to the engine tick. A preparation owns a local vector and lends it to a temporary loader while calling the same actor constructors. The normal loader has no collector. The vector is available to the preparation as soon as the constructor calls finish, including handles recorded on cache hits. This uses a single concrete constructor parameter with an optional exclusive borrow, with no dependency extraction operation on the actor interface. A second loader type and a generic loader trait would also express the distinction, but would propagate a generic parameter through the actor/parser call chain for the same two operations. Refill during activation uses the ordinary loader, and subsequent trigger preloads stay outside preparation dependencies.

[accepted] The author approved trying the single concrete loader. `EngineLoader` creates the ordinary loader once and transfers it into shared systems. `Engine` retains the system privately and maintains the cache at the same position in the tick. The resource system type is visible only inside the engine module. The final trigger now polls handles from `shared.resources.load(...)`. Actor constructors, including those reached during spare refill or a network spawn, receive the same mutable loader type. During initial construction its exclusive borrow records into the current preparation's vector, which the preparation then awaits.

Verified the revised interface on Linux in the uncommitted tree based on `75508b6`. The release workspace build, Linux bins check and Windows GNU bins check passed without compiler warnings. All three client bins passed 29 tests each, including shared failures collected by two preparations, headless media, world retention and trigger polling. The full workspace run encountered sandbox socket permission errors in 24 Matchbox tests and nine client transport tests per bin. With isolated loopback settings, the release client started the bundled map, completed local override A → external B → A and an external self-transition, and reported a malformed local override. The graphical run stopped at winit's `XOpenDisplayFailed`.

## 2026-11-24 — closed after author review

[accepted] The author reviewed the implementation and confirmed closure. Both loading paths use the same concrete loader. World preparation owns its dependency list and can await the handles directly once the loader releases its borrow.
