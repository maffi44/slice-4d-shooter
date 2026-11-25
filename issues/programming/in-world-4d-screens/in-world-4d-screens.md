# In-world 4D screens

- **Status:** Backlog — design pending
- **Priority:** High
- **Complexity:** Very complex
- **Created:** 2026-11-12
- **Stable address:** `issues/programming/in-world-4d-screens/`
- **Context:** [in-world-4d-screens-context.md](in-world-4d-screens-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Goal and scope

Display static textures or another world's output on an object in 4D space, with correct occlusion. A 2D HUD overlay does not satisfy the requirement. This remains separate from the engine refactoring.

## Proposals — not an implementation specification

Combine SDF depth output with a rasterized screen pass; decide W-thickness and correct cross-sections under arbitrary 4D rotation. Constant-size thickness and variable-size/double-pyramid ideas remain alternatives.

The render-thread texture resolver is deferred to this task. `WorldTextureId` and texture generations exist, but do not by themselves implement texture consumption. Resolve missing/headless/destroyed/resized textures and bind-group updates as part of the design.

Context preserves the author's geometry discussion, alternatives and open questions. Coordinate with [resource system](../resource-system/) and [GPU data grouping](../gpu-data-grouping/).
