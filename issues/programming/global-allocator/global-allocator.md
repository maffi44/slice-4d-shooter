# Global allocator: keep, replace or remove?

- **Status:** Open — decision or implementation pending
- **Priority:** Normal
- **Created:** 2026-11-13
- **Stable address:** `issues/programming/global-allocator/`
- **Context:** [global-allocator-context.md](global-allocator-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Problem

The project installs `blink_alloc::GlobalBlinkAlloc`, but never enables blink mode. Historical source investigation found inverted grow/shrink calls in direct-mode realloc in versions 0.3.1 and 0.4.0. Debug assertions exposed the issue; successful release runs do not establish correctness.

## Current decision and next step

Keep the existing interim configuration until the allocator decision: client uses it outside debug builds; game server excludes tests; lock stays at 0.3.1. Discuss removing it in favor of the standard allocator or choosing an alternative with evidence. No blanket dependency update.

Resolve allocator correctness in both client and server and record the choice and relevant verification. A new performance-instrumentation issue is not requested now. The detailed investigation, rejected update and alternatives are in context §§2–5.

Related: [engine refactoring](../engine-refactoring-world-isolation/).
