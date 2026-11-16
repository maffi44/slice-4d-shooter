# Context: Separate real elapsed time and simulation time if needed

[Current task](world-time-semantics.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

Review point 9. Author (2026-11-16) explained that the clamp served other purposes and proposed possibly exposing real and simulation time separately, retaining the 0.35 s rule for simulation.

Earlier rationale: a world simulated every Nth tick should use its own elapsed interval, with the existing large-step safeguard. See [refactoring context](../engine-refactoring-world-isolation/), §5.3 and §21.8. The safeguard conflicts with an unrestricted real-time claim for long intervals. [Engine tick rate](../engine-tick-rate/) controls scheduling and remains a separate issue. No standalone nested-world scenario task was requested.
