# Resource System

- **Status:** Backlog — design pending
- **Priority:** Normal
- **Created:** 2026-11-12
- **Stable address:** `issues/programming/resource-system/`
- **Context:** [resource-system-context.md](resource-system-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Goal

Provide typed path-based requests, async loading, shared resources, app-session caching and embedded-resource keys as part of `EngineSharedSystems`. Actors request resources; audio/UI and other systems create runtime elements from them.

## Current direction and open choices

Evaluate the existing `fyrox-resource` dependency before inventing another resource framework. Loader/device integration, handle semantics, cache keys, headless behavior and execution pool remain undecided. Version-specific research and rejected concerns are preserved in context.

The earlier reusable immutable `WorldDescriptor` design is implemented, but its world-loading contract is now under review: [async world preparation](../async-world-preparation/) has high priority. The author prefers investigating a special non-cloneable world resource over requiring `Clone` on every actor; this is a proposal, not an approved implementation. Distinguish shared handle clones from deep cloning actor state. Reconcile consume-once world preparation with reusable resource caching.

Complete the design decision and staged integration plan before a large implementation. Earlier synchronous request requirements remain historical context; heavy world preparation should be asynchronous under the newer loading discussion.
