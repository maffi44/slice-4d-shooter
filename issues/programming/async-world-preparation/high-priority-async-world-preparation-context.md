# Context: Prepare and validate worlds asynchronously

[Current task](high-priority-async-world-preparation.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

Review point 10. Author assigned high priority (2026-11-16).

Author's explanation: World versus WorldDescriptor was considered as the resource. A reusable World with actors appeared to require `Clone` on every actor; that requirement was undesirable, so the descriptor stores JSON and constructs fresh actors. But actor creation may itself request resources, causing real-time loading stalls. The author now wants all required data loaded asynchronously with world preparation and prefers exploring a special resource that its owner cannot clone.

Earlier arguments also concerned construction side effects and immutable sharing, not only cloning. Read [engine refactoring context](../engine-refactoring-world-isolation/), Part I “WorldDescriptor vs World”, §12, §20.1 and §21.6, then [Resource System context](../resource-system/). Reopen the earlier choice because storing raw descriptions leaves validation and resource work at activation; preserve its useful constraints.

Technical distinction raised during documentation: `Arc`/resource-handle cloning need not clone the contained actor, but sharing one mutable World is not the same as creating independent world instances. A non-cloneable prepared instance may be appropriate; it does not by itself solve side effects, thread affinity, or repeated-use semantics. Neither alternative has been implemented or selected as final architecture.
