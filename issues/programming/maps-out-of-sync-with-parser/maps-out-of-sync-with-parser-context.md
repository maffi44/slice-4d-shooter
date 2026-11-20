# Context: Maps out of sync with the parser

[Current task](maps-out-of-sync-with-parser.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

## Preserved history (before the 2026-11-16 documentation migration)

The following original issue is preserved in full. Its old filename rules, status summaries and “final” architecture headings describe their original rounds and are superseded by the current task and AGENTS.md where they disagree. Section numbers remain available for older references.

# Most maps don't load: map files are out of sync with the map parser

- **Created:** 2026-11-16
- **Status:** [open] open (found, not fixed, not discussed with the author yet)
- **Area:** programming (map loading, `src/engine/world/world_descriptor.rs`), level design (map files)
- **Related:** `issues/programming/engine-refactoring-world-isolation/` (found while verifying it: §24.9 round 7 and round 10, §24.10)

## 1. Problem

The map parser (`WorldDescriptor::load` → `parse_json_level` and the `parse_*` functions in `world_descriptor.rs`) requires properties that many map files in `src/assets/maps/` don't have. Such a map panics on load with `Wrong JSON map format: ...`. **This existed before the engine refactoring:** the parser evolved on the CTF map and the obstacle-course branch was merged (`8cf57cb`, `3802466`), but the older maps were never updated.

Today only `capture_the_flag_map` (the default level) is known to load.

Found so far (2026-11-13, by running `-l <map>`):

| Map | Missing / wrong |
|---|---|
| `obstacle_course_levels/*` (e.g. `level-1`, `level-2`) | top-level `it_is_2d_3d_example`; `allowed_dynamic_objects_in_bsp_tree` (some maps); in `final_trigger` actors: `async_preload` (and probably `async_load`) |
| `obstacle_course_map`, `map-3d-to-4d` | same: `it_is_2d_3d_example`, `allowed_dynamic_objects_in_bsp_tree`, `final_trigger.async_preload` |
| `map_2d_3d` | no `main_actor` |

The list isn't complete: after a missing property is added, the next missing one may show up. A map is only known to be correct once it loads and plays.

## 2. How to check a map without editing it

`WorldDescriptor::load` tries `./<name>.json` first, so you can run the client from another directory with a fixed copy: copy the map to a scratch directory, add the missing properties, run `<repo>/target/x86_64-unknown-linux-gnu/release/game-client -l <copy-name>` from that directory. This was used in round 10 to check that the hand-written raymarch shaders (`raymarch_shader_obstacle_course*.wgsl`) still compile. Their pipelines are created in `load`; the run then panicked later, in `create_actors`, on `final_trigger.async_preload`.

## 3. Options ([proposed], to discuss with the author)

- [proposed] Update the map files: add the missing properties with the values the old behaviour implied (`it_is_2d_3d_example: false`, `async_preload`/`async_load: false`, a permissive `allowed_dynamic_objects_in_bsp_tree`).
- [proposed] Or make the parser use defaults for optional properties (`unwrap_or(default)` instead of `expect`) so that older maps keep loading. This is simpler for content, but it hides mistakes in new maps.
- [open] Which maps are still wanted at all (obstacle course, puzzle levels, 2D/3D example)? Unwanted ones could move to `old_maps_archive/`.
- Note: the JSON keys contain typos (`frenel_color`, `sticiness_negative_*`, `trgger` in some names). They were deliberately not renamed in round 10. If the map format gets touched anyway, decide whether to fix them, which would mean updating all maps and the parser together.

## 4. Decision log

| # | Topic | Decision | Status |
|---|---|---|---|
| M1 | Fix maps vs. parser defaults | — | Open |
| M2 | Which maps to keep | — | Open |
