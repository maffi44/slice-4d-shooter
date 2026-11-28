# Maps out of sync with the parser

- **Status:** Closed
- **Priority:** Normal
- **Created:** 2026-11-16
- **Closed:** 2026-11-25
- **Resolution:** Obsolete — the author retired all affected maps.
- **Stable address:** `issues/programming/maps-out-of-sync-with-parser/`
- **Context:** [maps-out-of-sync-with-parser-context.md](maps-out-of-sync-with-parser-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Problem

Most maps lack fields currently required by the parser. Only `capture_the_flag_map` is known to load. This predates the engine refactoring; missing fields and the scratch-copy verification procedure are retained in context §§1–2.

## Resolution

[accepted] The client ships only `capture_the_flag_map`. The other 24 level files are preserved in `src/assets/maps/old_maps_archive/`. The historical schema investigation is recorded in the context.
