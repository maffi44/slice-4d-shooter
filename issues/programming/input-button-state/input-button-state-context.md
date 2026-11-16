# Context: Preserve short presses and multiple input bindings

[Current task](input-button-state.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

Review point 4. On 2026-11-16 the author requested an issue but explicitly said he had not checked the behavior. Do not label this user-verified.

Removing `Arc<Mutex>` was an accepted simplification because input is collected on one thread; see [refactoring context](../engine-refactoring-world-isolation/), §8 and §24.9. That ownership decision remains useful and does not require restoring locks to fix event semantics.
