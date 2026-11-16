# Engine tick scheduling

- **Status:** Open — decision or implementation pending
- **Priority:** Normal
- **Created:** 2026-11-13
- **Stable address:** `issues/programming/engine-tick-rate/`
- **Context:** [engine-tick-rate-context.md](engine-tick-rate-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Problem and goal

Both windowed and headless loops schedule from `now + period`, accumulating wake-up lateness. Target an average of 60 ticks/second when workload and scheduling permit it; exact OS wake-ups are not guaranteed.

## Proposed approach — not yet approved

Advance deadlines relative to a fixed timeline, with a shared scheduling calculation for both loops. Decide bounded catch-up versus resynchronization after stalls. Measure before adding timer changes or spinning.

Verify elapsed time, tick counts and lateness on Linux and Windows. The server's separate sleep-after-work loop is related but needs an explicit scope decision. Keep scheduling separate from [world clock semantics](../world-time-semantics/).

Context contains the original motivation, proposed formulas, overload questions and discussion. No new duplicate tick-rate issue is needed.
