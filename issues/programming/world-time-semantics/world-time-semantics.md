# Separate real elapsed time and simulation time if needed

- **Status:** Open — not implemented
- **Priority:** Normal
- **Created:** 2026-11-16
- **Stable address:** `issues/programming/world-time-semantics/`
- **Context:** [world-time-semantics-context.md](world-time-semantics-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Task

`WorldTimeSystem` clamps intervals over 0.35 s to roughly one nominal tick. A rarely simulated world therefore does not necessarily advance at real-time speed.

Discuss two clocks: real elapsed time for consumers that need it, and simulation time using the existing approximation. The clamp had a deliberate purpose; do not simply remove it. Define pauses, sparse simulation, first tick and resume behavior, and distinguish world time from session/server time.

Completion: the chosen contract and use cases are explicit, callers use the appropriate clock, and documentation no longer promises unconditional real-time progress. A second public clock is a proposal, not a settled requirement.
