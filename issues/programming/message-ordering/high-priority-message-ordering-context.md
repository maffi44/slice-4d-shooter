# Context: Define and preserve message ordering

[Current task](high-priority-message-ordering.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

Review point 3. Author agreed and assigned high priority (2026-11-16).

Earlier decision: keep LIFO because actor logic treats a tick as simultaneous, and `pop()` is efficient. Context: [engine refactoring](../engine-refactoring-world-isolation/), §19 round 6, §23 and §21.4. That argument does not hold for successive state assignments; retaining capacity is independent of FIFO versus LIFO. A `VecDeque` or reusable sequential buffers are candidates, not a chosen design.

Player doll timestamps already reject some old updates; that does not protect score/flag state generally. Preserve the previous decision as history, with this concrete reason for reopening it.
