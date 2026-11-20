# Context: Close connections when their owner is removed

[Current task](session-connection-lifetime.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

Review point 2. The author confirmed the finding and called it very high priority on 2026-11-16. No implementation mechanism was selected.

The original design deliberately made transport application-wide and sessions per-world. Rendering already uses an owner lifetime token, but network handles currently do not express ownership. Read [refactoring context](../engine-refactoring-world-isolation/), §13 and §21.9, before changing these responsibilities. Keep transport protocol-agnostic; cleanup must also work for arbitrary nested worlds.
