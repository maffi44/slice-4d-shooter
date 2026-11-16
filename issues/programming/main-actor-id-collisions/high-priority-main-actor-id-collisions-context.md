# Context: Preserve main_actor_id during actor ID collisions

[Current task](high-priority-main-actor-id-collisions.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

Review point 6. Author called this very important and high priority (2026-11-16).

The network peer ID may replace the main actor's initial ID; collision handling is deliberate, not merely a theoretical random-ID concern. Context: [engine refactoring](../engine-refactoring-world-isolation/), §20.2 and §24.7. Keep `ChangeMyId` semantics unless there is a separately agreed reason to change them.
