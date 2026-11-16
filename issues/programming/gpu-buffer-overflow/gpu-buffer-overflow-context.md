# Context: Protect GPU upload buffers from overflow

[Current task](gpu-buffer-overflow.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

Review point 5. Author agreed (2026-11-16), stressed the temporary nature of the current upload structure, and requested long-term overflow protection rather than treating existing array sizes as permanent architecture.

The review identified possible CPU indexing panics when capacities are exceeded; occurrence in gameplay has not been established here. Coordinate with [GPU grouping](../gpu-data-grouping/), but capacity safety and cross-buffer consistency are distinct concerns. No immediate renderer redesign was authorized.
