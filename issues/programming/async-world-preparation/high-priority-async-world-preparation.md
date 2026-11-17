# Prepare and validate worlds asynchronously

- **Status:** Open — not implemented
- **Priority:** High
- **Created:** 2026-11-16
- **Stable address:** `issues/programming/async-world-preparation/`
- **Context:** [high-priority-async-world-preparation-context.md](high-priority-async-world-preparation-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Task

Heavy data/resource loading and validation should complete during asynchronous world preparation, rather than unexpectedly when actors are instantiated on the main thread. Today `WorldDescriptor` stores actor JSON, and `create_actors()` parses it later; future actor resource requests could also block activation.

## Alternatives to discuss

1. Require actors to implement `Clone` and prepare reusable actor prototypes.
2. Introduce a special non-cloneable resource for a prepared `World`, or a descriptor containing ready actors, transferred to its owner. **Author's preferred direction to investigate**, not yet an approved implementation.

Separate preparation from activation side effects: audio, UI, connections, render registration and callbacks must not run merely because a world is preloaded. Establish thread/Send constraints and whether CPU preparation, GPU work and activation require different execution contexts.

Distinguish cloning a shared resource handle from deep-cloning mutable actors. A consume-once prepared world also needs a policy for repeated requests/reloads and caching; this does not automatically require `Clone` on `Actor`.

Completion: malformed actor data and resource failures surface during loading, required resources are ready before activation, activation performs no hidden heavy loading, and ownership/reuse/error behavior is documented and verified. Coordinate [Resource System](../resource-system/) and [map compatibility](../maps-out-of-sync-with-parser/).

Related startup discussion: [nonblocking engine startup](../nonblocking-engine-startup/) covers event-loop ownership, GPU prerequisites and readiness notifications. This task retains responsibility for complete resource preparation and activation semantics.
