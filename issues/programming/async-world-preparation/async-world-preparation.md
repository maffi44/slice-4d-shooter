# Prepare and validate worlds asynchronously

- **Status:** Done — accepted by the author on 2026-11-20
- **Priority:** High
- **Created:** 2026-11-16
- **Closed:** 2026-11-20
- **Resolution:** Done
- **Implementation commit:** `858c5ae` (reviewed by the author)
- **Stable address:** `issues/programming/async-world-preparation/`
- **Context:** [async-world-preparation-context.md](async-world-preparation-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Task

Heavy data/resource loading and validation should complete during asynchronous world preparation, rather than unexpectedly when actors are instantiated on the main thread. Previously, actor JSON was interpreted during activation.

## Accepted implementation

The first actor set is built during asynchronous map preparation. Each constructor acquires its own required handles. After they are ready, the descriptor can give its actors to an independent world and synchronously build the next spare from its retained JSON and warm resource cache. Render registration, actor callbacks and initial messages belong to activation. The public entry point is `descriptor.give_me_world(config, shared)`. See the [Resource System](../resource-system/) implementation history.

Review covers malformed map diagnostics, independent world instances, resource readiness and absence of loading-time gameplay effects. Existing incompatible maps are tracked under [map compatibility](../maps-out-of-sync-with-parser/).
