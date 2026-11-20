# GPU upload grouping and coherent data

- **Status:** Backlog — long-term
- **Priority:** Long-term
- **Complexity:** Complex
- **Created:** 2026-11-16
- **Stable address:** `issues/programming/gpu-data-grouping/`
- **Context:** [gpu-data-grouping-context.md](gpu-data-grouping-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Task

Group compatible GPU uploads into larger blocks where useful. Current `WorldRenderSystem::update` issues multiple writes while the render thread can submit between them, so a frame can combine metadata and arrays from different simulation ticks.

The author assigns this long-term priority: expected visual impact is small, but it has not been measured. Do not assume every world or pipeline can use one universal block. Some data is shared across pipelines; a world may have one or several main pipelines with different data requirements.

Investigate grouping by world/pipeline and shared data usage, preserving relevant limits, layouts and consumers. Compare performance before/after. Define which data must be coherent and what cross-block inconsistency remains; grouping alone does not guarantee whole-frame atomicity.
