# Resource System

- **Status:** Implemented — awaiting author review
- **Priority:** Normal
- **Created:** 2026-11-12
- **Stable address:** `issues/programming/resource-system/`
- **Context:** [resource-system-context.md](resource-system-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Accepted design

From an actor’s perspective, request a resource, inspect readiness and use the ready value. Loading, caching and execution belong to the resource system. Consumers choose what pending means in their own domain. Audio can accept a pending buffer, while a final trigger can defer its transition to the next tick. Errors terminate the current attempt. Keep the same handle contract across asset types and specialize only the work required to produce their data.

One application-wide `ResourceSystem` owns Fyrox 0.36.2's manager. Typed requests return a shared handle immediately, including during loading. Callers can inspect its state or await the result. A session request pins even a previously requested resource until shutdown. Ordinary requests follow Fyrox's unused-resource expiry.

Maps share one descriptor per logical key. Startup registers the initial map under the same key as transitions. A descriptor owns parsed actor JSON, prepared geometry/pipelines and one spare set of live, unactivated actors. `give_me_world(config, shared)` synchronously replenishes the spare, transfers the previous actors and activates the world. Each caller owns an independent simulation.

Actors and devices store typed handles directly in named fields and request their own embedded WAV by path. Initial preparation waits on a per-load list of the resources actually requested. Headless skips audio. Dynamic effects can receive the specific buffers they need from the object spawning them. A final trigger requests its destination on addition when preload is enabled, supporting self-transitions without a cycle of ready-state dependencies. The legacy boolean `async_load` is accepted and has no effect on tick scheduling.

## Scope and review

The approved integration covers gameplay sounds and world descriptors. UI and UI textures are reserved for the separate UI redesign. Network runtime isolation is a separate scheduling question.

Review criteria include concurrent request reuse, session retention, independent actor instances, asynchronous failure propagation, working headless and graphical clients, and documented map compatibility. See the [implementation and verification log](resource-system-context.md).
