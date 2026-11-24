# Unify typed resource requests across actors and engine startup

- **Status:** Done
- **Priority:** Normal
- **Created:** 2026-11-23
- **Closed:** 2026-11-24
- **Resolution:** Done
- **Context:** [Caller examples, existing behavior and design choices](unified-resource-requests-context.md)

## Goal

[accepted] Design the smallest common request that serves a sound in an actor, a destination level in a trigger and the initial world in `EngineLoader`. The caller supplies a key and retention, and receives a typed handle immediately. Rust can infer the resource type from the destination field or the caller can specify it explicitly. The resource system owns lookup, sharing, background preparation and publication of success or failure. Introduce a type-specific public operation only when a concrete caller requirement justifies it.

## Approved design and implementation

[accepted] The ordinary `load<T>` returns a typed handle. The media call skips headless requests and otherwise delegates to the ordinary call. Required GPU-only resources return a terminal error in headless. Caller-selected retention and each world's dependency collection apply through the shared path.

[accepted] CLI and trigger map names prefer the same ordered local JSON candidates, then the embedded catalog. Other file resources prefer the catalog. The engine starts its initial world with `WhileUsed` before device discovery completes. A future caller can request a resident menu with `Session`. See the [architecture reference](../../../docs/engine-architecture.md) for caller examples and the extension steps for a file-backed texture.

## Review follow-up

[accepted] Gameplay receives a concrete `ResourceLoader` exposing only `load` and `load_media`. Each world preparation lends its dependency list to the loader during construction, then awaits it. The engine owns cache maintenance. See the [review discussion](unified-resource-requests-context.md#2026-11-24--review-of-actor-access-and-preparation-ownership).

## Completion criteria

1. Agree on the caller examples in the context before implementation. Trace those same examples through the final code. Document the common contract and any justified exception in the [engine architecture reference](../../../docs/engine-architecture.md).
2. A sound and a map can each be requested with caller-selected retention. Requests for the same resolved asset share pending and ready work. An async caller can await readiness; a trigger can poll without blocking its tick. Missing or malformed resources report failure.
3. Embedded and external inputs use the same type-specific preparation after byte acquisition. Verify a bundled map, a local override and an external-only map selected by `-l` against the agreed lookup rules.
4. Startup uses the resource system and completes before the first simulation tick. Preserve overlap between CPU map preparation and GPU initialization, cancellation, and headless loading without sound decoding or GPU work.
5. Verify session retention across menu → game → menu using a representative map fixture. Returning can reuse the prepared descriptor and create an independent world instance. Also verify ordinary expiry, actor dependencies and transitions, including self-transitions.
