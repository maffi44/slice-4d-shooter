# Context: GPU upload grouping and coherent data

[Current task](gpu-data-grouping.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

Review point 1: the review proposed coherent snapshots/publication because thread-safe `write_buffer` calls do not make a batch atomic relative to `submit`.

Author response (2026-11-16): agreed with the concern but lowered priority; pursue larger GPU blocks as long-term work. A brief mismatch is expected to be visually unobtrusive. Full consolidation is neither required nor assumed feasible because shared and pipeline-specific data coexist.

Technical qualification: the interval depends on submission/tick timing and stalls, not a guaranteed fraction of a second. One larger block can improve coherence for that block without solving coherence between all blocks. Choose a design only after examining usage and performance. Related: [overflow protection](../gpu-buffer-overflow/); [refactoring context](../engine-refactoring-world-isolation/), §9 and §24.9.
