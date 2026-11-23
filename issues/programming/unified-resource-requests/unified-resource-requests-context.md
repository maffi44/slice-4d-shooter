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
