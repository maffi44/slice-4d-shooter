# Maps out of sync with the parser

- **Status:** Open — decision or implementation pending
- **Priority:** Normal
- **Created:** 2026-11-16
- **Stable address:** `issues/programming/maps-out-of-sync-with-parser/`
- **Context:** [maps-out-of-sync-with-parser-context.md](maps-out-of-sync-with-parser-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Problem

Most maps lack fields currently required by the parser. Only `capture_the_flag_map` is known to load. This predates the engine refactoring; missing fields and the scratch-copy verification procedure are retained in context §§1–2.

## Decisions needed

Choose supported maps, update content or supply intentional defaults for optional fields, and decide whether obsolete maps belong in the archive. Do not silently rename JSON keys or change gameplay values.

A supported map must load through actor creation and run its intended transitions. Coordinate with [async world preparation](../async-world-preparation/), which addresses the loading boundary rather than choosing map content. No separate test-suite task is requested.
