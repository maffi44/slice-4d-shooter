# Unify typed resource requests across actors and engine startup

- **Status:** Open — design pending
- **Priority:** Normal
- **Complexity:** Complex
- **Created:** 2026-11-23
- **Context:** [Caller examples, existing behavior and design choices](unified-resource-requests-context.md)

## Goal

[accepted] Design the smallest common request that serves a sound in an actor, a destination level in a trigger and the initial world in `EngineLoader`. The caller supplies a key and retention, and receives a typed handle immediately. Rust can infer the resource type from the destination field or the caller can specify it explicitly. The resource system owns lookup, sharing, background preparation and publication of success or failure. Introduce a type-specific public operation only when a concrete caller requirement justifies it.

## Design decisions to make with the author

- Choose how a logical key selects embedded bytes or an external file, including ambiguous keys and the precedence of local map overrides. Prefer one request; consider an explicit path variant if the caller needs to select a particular file.
- Decide how to express optional sound in headless mode and how to collect a world's preparation dependencies through the common request.
- Agree on startup ordering and who chooses retention for the initial world. A future menu should support `Session`; the initial world can also be an arbitrary map supplied by `-l`.
- Demonstrate how a future file-backed texture would fit the contract. Texture implementation and dynamic render targets belong to their own work.

## Completion criteria

1. Agree on the caller examples in the context before implementation. Trace those same examples through the final code. Document the common contract and any justified exception in the [engine architecture reference](../../../docs/engine-architecture.md).
2. A sound and a map can each be requested with caller-selected retention. Requests for the same resolved asset share pending and ready work. An async caller can await readiness; a trigger can poll without blocking its tick. Missing or malformed resources report failure.
3. Embedded and external inputs use the same type-specific preparation after byte acquisition. Verify a bundled map, a local override and an external-only map selected by `-l` against the agreed lookup rules.
4. Startup uses the resource system and completes before the first simulation tick. Preserve overlap between CPU map preparation and GPU initialization, cancellation, and headless loading without sound decoding or GPU work.
5. Verify session retention across menu → game → menu using a representative map fixture. Returning can reuse the prepared descriptor and create an independent world instance. Also verify ordinary expiry, actor dependencies and transitions, including self-transitions.
