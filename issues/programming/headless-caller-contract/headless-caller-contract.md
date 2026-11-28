# Unify graphical and headless APIs for gameplay

- **Status:** Open — design pending
- **Priority:** Normal
- **Complexity:** Complex
- **Created:** 2026-11-28

## Scope and decisions

[accepted] Actors should request media, construct UI, and use world and shared systems through the same API in graphical and headless sessions. Preserve simulation, scanner messages and all state exposed to gameplay. Headless sessions should skip driver access, presentation, playback, media reads and per-tick work that has no observable result.

Today `load_media` and the world's optional renderer expose the distinction to callers. Meanwhile the CTF player's opacity updates materialize widgets and parents in each headless world. Review this case together with the [Fyrox HUD migration](../fyrox-ui/).

[open] Specify handle readiness, failures, lifetimes and which data a caller can inspect. A resource with an optional internal backend payload is one candidate. Resolve which widget state must exist when gameplay queries a hierarchy or property. Distinguish media needed only for output from data also needed by simulation.

## Completion criteria

- Agree on examples for a scanner image, a sound request and a queryable widget subtree. Run the same actor code in both modes and compare observable state, messages and completion behavior.
- Implement the contract across resource loading, rendering, audio and UI. Verify that headless sessions avoid output-only asset reads, device initialization, GPU allocation and draw submission.
- Verify which HUD allocations and tick updates can be skipped in multiple headless worlds. Preserve the query results and hierarchy changes promised by the agreed contract. Document the API and remaining per-world costs.
