# Context: Global allocator: keep, replace or remove?

[Current task](global-allocator.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

## Preserved history (before the 2026-11-16 documentation migration)

The following original issue is preserved in full. Its old filename rules, status summaries and “final” architecture headings describe their original rounds and are superseded by the current task and AGENTS.md where they disagree. Section numbers remain available for older references.

# Global allocator: do we need a custom one at all?

**Status:** [open] open, the next discussion stage after the engine refactoring (`issues/programming/engine-refactoring-world-isolation/`).
**Area:** programming (client, game server).

## 1. Context / motivation

Slice is a fast competitive shooter. A core principle of the project (`AGENTS.md`, core principle 4) is: **no unnecessary memory allocations or reallocations — even with a fast allocator**. Hot paths (every tick / frame) should not allocate; buffers are reused (`pop()`/`drain(..)` instead of `mem::take`, iterators instead of fresh `Vec`s, `clear()` + reuse).

Separately, the client and the game server install a custom global allocator, `blink_alloc::GlobalBlinkAlloc` (added in `a06764e`, 2024-12-18: "added global allocator for game server and client"):
- client: `#[cfg(not(debug_assertions))]` (disabled in debug builds since `d3cda7e`, 2025-10-03);
- game server: always, except `#[cfg(not(test))]` since the engine refactoring (round 7–8).

The question: does the custom allocator give us anything, is it correct, and should we keep it, replace it, or drop it?

## 2. Findings (2026-11-13, engine refactoring round 8)

How it came up: `cargo test --workspace` failed — the `game_server` test binary panicked at start-up:

```
allocator-api2-0.2.21/src/stable/alloc/system.rs:57:9:
`new_layout.size()` must be smaller than or equal to `old_layout.size()`
```

1. **Not caused by `cargo update`.** The author suspected the panics started after a `cargo update`. But `game_server` has no own `Cargo.lock` (it's a workspace member, the root lock is shared), and the lock had **blink-alloc 0.3.1 / allocator-api2 0.2.21 in every commit since 2025** (checked from `16b20a0` to `f3e2128`). The test panic also reproduced on a clean `HEAD` worktree.
2. **Updating doesn't help.** `cargo update -p blink-alloc` → 0.4.0 (+ allocator-api2 0.4.0): same panic. The lock was rolled back to 0.3.1.
3. **Root cause — a bug in blink-alloc** (same code in 0.3.1 and 0.4.0, `src/global/sync.rs`, `State::resize`):
   ```rust
   match self.enabled {
       true => self.blink.resize(ptr, old_layout, new_layout),
       false => {
           cold();
           if old_layout.size() >= new_layout.size() {
               self.blink.inner().grow(ptr, old_layout, new_layout)     // shrinking → calls grow
           } else {
               self.blink.inner().shrink(ptr, old_layout, new_layout)   // growing → calls shrink
           }
       }
   }
   ```
   The condition is inverted. In debug builds allocator-api2's `debug_assert!` in `shrink` fires (the panic above). In release builds the assertion is compiled out, but `shrink` then gives the optimizer a false `assume(new_size <= old_size)` — undefined behaviour in principle; in practice `realloc` still receives the right size, which is why release builds seem to work.
4. **The game never enables blink mode.** `GlobalBlinkAlloc` is created in *direct mode* (`enabled: false`); blink (arena) mode is switched on only by `unsafe { GLOBAL_ALLOC.blink_mode() }`, which the code never calls. In direct mode every allocation is passed to the system allocator (`std::alloc::System`). So **today the custom allocator gives no speed-up at all** — it's the system allocator plus the realloc bug above. This also explains `d3cda7e` ("disabled global allocator in debug build"): the same debug panic.
5. **Real blink mode isn't a drop-in speed-up.** In blink mode allocations come from an arena and memory is reclaimed only by `reset()`, which is allowed only when nothing allocated in blink mode is still alive. That fits per-frame scratch data with a clear reset point, not long-lived game state (actors, worlds, network buffers) — using it would need a redesign of where allocations happen.

## 3. Discussion

- **Author (round 8):** «blink-alloc. Я не уверен, что он не работает. Но если ты проверял исходники и там есть проблемы, то, может быть, действительно нет смысла его использовать. Может быть, стоит поискать альтернативы blink-alloc. В общем, про аллокаторы давай поговорим чуть позже — это будет следующая стадия обсуждений.»
- **Claude (round 8):** the panic is not from `cargo update` (same versions for a year) and not fixed by 0.4.0; it's an inverted `grow`/`shrink` condition in blink-alloc's direct mode; and since the game never calls `blink_mode()`, blink-alloc currently just forwards to the system allocator. [proposed] Recommendation: remove blink-alloc from client and server — the system allocator is what actually runs now, and the "no allocations" principle is achieved in code, not by the allocator. Interim state: `game_server`'s global allocator is `#[cfg(not(test))]` with a comment explaining the bug; the client keeps `#[cfg(not(debug_assertions))]`.

## 4. Questions for the next stage

- [open] Do we want a custom global allocator at all, given that hot paths are meant to be allocation-free?
- [open] If yes, which one and why — measured, not assumed. Candidates to evaluate: `mimalloc`, `jemalloc` (`tikv-jemallocator`; not on Windows MSVC), `snmalloc`, `rpmalloc`; or blink-alloc fixed upstream (report/PR the inverted condition) and used in real blink mode only for a scoped per-frame arena.
- [open] How to measure: allocation counts per tick/frame (a counting allocator in debug builds), frame-time percentiles in the client, tick time on the server under load.
- [open] Server vs client: different allocation patterns (server: many connections, long uptime → fragmentation matters more).
- [proposed] Possible first step regardless of the choice: a debug-only counting allocator that reports allocations per tick, to find hot-path allocations to remove (core principle 4).

## 5. Decision log

| # | Topic | Decision | Status |
|---|---|---|---|
| A1 | Cause of the test panic | blink-alloc direct-mode `resize` bug (inverted grow/shrink), present in 0.3.1 and 0.4.0; not caused by `cargo update` | [accepted] verified |
| A2 | Lock file | stays on blink-alloc 0.3.1 until the allocator decision | Accepted |
| A3 | Interim | `game_server` global allocator `#[cfg(not(test))]`; client unchanged | Accepted |
| A4 | Keep / replace / remove the custom allocator | — | [open] next stage |
