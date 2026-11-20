# Split mixed responsibilities into logical submodules

- **Status:** Open — not implemented
- **Priority:** Normal
- **Complexity:** Complex
- **Created:** 2026-11-16
- **Stable address:** `issues/programming/logical-module-splitting/`
- **Context:** [logical-module-splitting-context.md](logical-module-splitting-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Task

Inspect large modules such as MainPlayer, NetSession, map parsing and shader/BSP generation for independent responsibilities. Split along logical boundaries, retaining clear ownership, discoverability and established public behavior.

File length alone is not a reason to split. Avoid arbitrary line limits, forwarding-only wrappers and module fragmentation. Possible boundaries include connection state versus protocol translation, and BSP construction versus WGSL emission; these are candidates, not a mandated layout.

Agree each substantial decomposition before implementing; keep changes staged and preserve capabilities. Completion: chosen responsibilities have clear homes, imports/docs are updated and behavior/builds remain intact. The general rule is now in AGENTS.md.
