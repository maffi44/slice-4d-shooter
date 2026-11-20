# Context: Prepare and validate worlds asynchronously

[Current task](async-world-preparation.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

Review point 10. Author assigned high priority (2026-11-16).

Author's explanation: World versus WorldDescriptor was considered as the resource. A reusable World with actors appeared to require `Clone` on every actor; that requirement was undesirable, so the descriptor stores JSON and constructs fresh actors. But actor creation may itself request resources, causing real-time loading stalls. The author now wants all required data loaded asynchronously with world preparation and prefers exploring a special resource that its owner cannot clone.

Earlier arguments also concerned construction side effects and immutable sharing, not only cloning. Read [engine refactoring context](../engine-refactoring-world-isolation/), Part I “WorldDescriptor vs World”, §12, §20.1 and §21.6, then [Resource System context](../resource-system/). Reopen the earlier choice because storing raw descriptions leaves validation and resource work at activation; preserve its useful constraints.

Technical distinction raised during documentation: `Arc`/resource-handle cloning need not clone the contained actor, but sharing one mutable World is not the same as creating independent world instances. A non-cloneable prepared instance may be appropriate; it does not by itself solve side effects, thread affinity, or repeated-use semantics. Neither alternative has been implemented or selected as final architecture.

## 2026-11-16 — Startup follow-up

The author reopened application startup after accepting initial refactoring commit `caea9ec`. [Startup context](../nonblocking-engine-startup/) records research on parallel CPU/GPU preparation, dependent pipeline creation and event-driven readiness. The two tasks share the preparation boundary; no resource ownership alternative above has been silently selected.

## 2026-11-19 — actor descriptors approved

[accepted] Reusable factories and preloaded dependencies supersede the consume-once world and cloneable-actor proposals. See the [Resource System](../resource-system/) implementation history.

## 2026-11-20 — spare actor set replaces factories

[accepted] Author review superseded the actor descriptor factories. The map keeps parsed actor inputs and a single prepared set. Its public method owns the refill and activation sequence. See the [Resource System](../resource-system/) review and verification log.

## 2026-11-20 — Closed

[accepted] The author confirmed that the review was done and the implementation committed in `858c5ae`; the task is closed as Done. Later changes go to new or existing open tasks.
