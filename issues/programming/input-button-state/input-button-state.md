# Preserve short presses and multiple input bindings

- **Status:** Open — not implemented
- **Priority:** Normal
- **Complexity:** Moderate
- **Created:** 2026-11-16
- **Stable address:** `issues/programming/input-button-state/`
- **Context:** [input-button-state-context.md](input-button-state-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Task

Investigate event accumulation in `InputSystem`: release clears `just_pressed` before the next tick, release of one binding clears a shared action while another may remain held, and repeated presses increment a counter without reliable per-button state.

The author has not reproduced this; distinguish static-review findings from runtime verification. Reproduce press/release within one tick, multiple bindings and key repeat. Fix button state tracking and preserve press edges until consumption without an unrelated input rewrite.

Completion: short presses are observed, releasing one held binding does not release another, repeated events do not corrupt state. Source: `src/engine/shared_systems/input.rs`.
