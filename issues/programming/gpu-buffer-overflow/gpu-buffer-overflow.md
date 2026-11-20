# Protect GPU upload buffers from overflow

- **Status:** Backlog — long-term
- **Priority:** Long-term
- **Complexity:** Moderate
- **Created:** 2026-11-16
- **Stable address:** `issues/programming/gpu-buffer-overflow/`
- **Context:** [gpu-buffer-overflow-context.md](gpu-buffer-overflow-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Task

Design explicit capacity and overflow behavior for data sent to GPU. The current rendering data structure was a temporary solution, not a final format.

`dynamic_render_data.rs` fills fixed arrays (256 spherical areas/beams, 16 player forms) with unchecked increasing indices. Inspect other GPU arrays and matching metadata/shader limits as well.

Choose a durable structure or policy: bounded production, prioritization, suitable resizing or another justified approach. Do not silently discard gameplay-relevant geometry. Completion: excess input is handled deliberately, metadata never exceeds uploaded capacity, CPU/GPU layout assumptions agree and performance implications are assessed.
