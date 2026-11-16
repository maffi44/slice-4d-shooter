# Context: Engine tick scheduling

[Current task](engine-tick-rate.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

## Preserved history (before the 2026-11-16 documentation migration)

The following original issue is preserved in full. Its old filename rules, status summaries and “final” architecture headings describe their original rounds and are superseded by the current task and AGENTS.md where they disagree. Section numbers remain available for older references.

# Engine tick rate: an exact average tick period of 16.(6) ms

- **Created:** 2026-11-13
- **Status:** ❓ open (not started)
- **Area:** programming (engine main loop, both windowed and headless)
- **Related:** `issues/programming/engine-refactoring-world-isolation/` (the main loop moved to `src/engine/main_loop.rs` in round 7), `issues/programming/network-authoritative-server-and-transport/` (server tick, see §5)

## 1. Motivation (from the author)

The engine tick rate is not accurate: the current way of scheduling ticks is "far from ideal". It's acceptable as a first approximation, but the tick period should be **exactly 16.(6) ms on average** (60 ticks per second). This is needed both for the headless engine and for the normal (windowed) engine.

The author already had such an implementation once, but it was lost: it existed before the first commit, so it isn't in the git history. He remembers roughly that it computed some **average value**.

Why it matters: game logic, physics and the network rate are tuned for 60 Hz. A tick that is systematically longer than 16.(6) ms means fewer ticks per second than intended: the simulation and the packets sent to the server run slower than the server expects. The client and the server have to agree on the tick rate (see §5).

## 2. Current implementation (2026-11-13)

The target period is `GlobalTimeSystem::target_frame_duration` = `1 / 60` s (`GlobalTimeSystem::new(60)` in `src/engine/shared_systems.rs`).

**Windowed** (`src/engine/main_loop.rs`, `MainLoop::new_events`):
```rust
StartCause::ResumeTimeReached {..} => {
    event_loop.set_control_flow(ControlFlow::WaitUntil(
        Instant::now() + engine.shared.time.target_frame_duration
    ));
    engine.tick();
    ...
}
```

**Headless** (`Engine::run_headless`):
```rust
let next_tick = Instant::now() + self.shared.time.target_frame_duration;
self.tick();
std::thread::sleep(next_tick.saturating_duration_since(Instant::now()));
```

**Problem:** in both cases the next deadline is computed **from the moment of wake-up** (`Instant::now() + period`), not from the previous deadline. Every wake-up comes late, because of OS timer granularity, scheduler latency and event-loop overhead. Typical lateness: tens of µs to ~1 ms on Linux, and up to ~1–15.6 ms on Windows with the default timer resolution. That lateness is **never compensated**, so:
- the real period = 16.(6) ms + the wake-up lateness, i.e. it is always ≥ 16.(6) ms;
- the error accumulates: the average tick rate is systematically below 60 Hz, and on Windows it can be noticeably below it.

In windowed mode a tick that takes longer than the period (e.g. a hitch) just shifts the whole schedule. That is acceptable, but it should be a deliberate policy (see §3).

## 3. Proposed approach (💡, to discuss)

💡 **A fixed timeline instead of "now + period".** Keep the schedule start `t0` and the tick number `n`. The deadline of tick `n` is `t0 + n * period`. To avoid rounding drift, compute it exactly from integers, e.g. `t0 + Duration::from_nanos(n * 1_000_000_000 / 60)` (16 666 666.(6) ns can't be stored exactly as one `Duration`; computing from `n` keeps the error below 1 ns forever). Lateness at one wake-up then automatically shortens the wait before the next tick, so **the average period is exactly 16.(6) ms**, and individual periods jitter around it.

This is probably the same idea as the author's lost "average value" implementation, just phrased differently. Correcting the next sleep by the measured average error converges to the same result. The fixed timeline is simpler and has no tuning parameters.

Things to decide:
- ❓ **Falling behind** (a tick longer than the period, a debugger pause, window dragging on Windows, loading): run catch-up ticks back to back, or re-sync the timeline (`t0 = now`, `n = 0`) when we are more than N periods late? 💡 Proposal: allow a small catch-up (1–2 ticks), otherwise re-sync, so that after a long pause the simulation doesn't spin hundreds of ticks. World delta time already falls back to one nominal frame for huge deltas (`MAX_DELTA` / `FALLBACK_DELTA` in `time.rs`).
- ❓ **Precision of the wait itself.** The fixed timeline fixes the *average*, but each tick's jitter still depends on the timer:
  - Windows: `timeBeginPeriod(1)` or a high-resolution waitable timer (`CREATE_WAITABLE_TIMER_HIGH_RESOLUTION`); check what winit 0.30's `ControlFlow::WaitUntil` already uses internally;
  - optionally sleep until ~1 ms before the deadline and spin (busy-wait / `yield`) for the rest. This costs a little CPU but gives sub-ms jitter.
  - Measure before choosing.
- ❓ **One implementation for both loops.** 💡 Put the schedule into `GlobalTimeSystem` (e.g. `next_tick_deadline()` + advancing the timeline in `end_of_frame`). The headless loop sleeps until the deadline; the windowed loop passes it to `ControlFlow::WaitUntil`. Then both modes share exactly the same logic.
- ❓ Does winit deliver `ResumeTimeReached` on time while there are many window events (mouse movement at a high polling rate)? Verify that input events don't delay ticks.

## 4. How to verify

- Log the statistics: tick count over e.g. 60 s / elapsed time → must be 60.00 Hz; also the min/avg/max period and the lateness distribution. `GlobalTimeSystem` already has `frame_counter` and `average_frame_duration`, which are unused today.
- Check on Linux and on Windows, windowed and `--headless`.

## 5. Out of scope / related

- The **game server** has its own loop with `tokio::time::sleep(Duration::from_millis(16))` after each iteration (`game_server/src/main.rs`): its period is 16 ms + processing + lateness, not 16.(6) ms. It has the same kind of problem and could use the same fixed-timeline approach (`tokio::time::interval` with `MissedTickBehavior` is one option there). Decide together with this issue or in the network issue.
- The render loop (`RenderScheduler`, its own thread, ~125 FPS target, skips frames while the GPU is busy) is independent of the engine tick and isn't part of this issue.

## 6. Discussion

- **Author (2026-11-13):** the tick rate isn't quite accurate; the current way is not ideal, fine as a first approximation, but the period should be exactly 16.(6) ms on average, for both the headless and the normal engine. He had such an implementation before the first commit (lost); as he remembers, it computed some average value.
- **Claude (2026-11-13):** the root cause is that the deadline is `now + period` computed after wake-up, so wake-up lateness accumulates and the average period is always longer than 16.(6) ms. 💡 Proposal: a fixed timeline (`t0 + n * period`, computed exactly from integers), shared by both loops via `GlobalTimeSystem`, plus a policy for falling behind; then measure the per-tick jitter and decide whether a high-resolution timer or a short final spin is needed.

## 7. Decision log

| # | Topic | Decision | Status |
|---|---|---|---|
| T1 | Target | average tick period exactly 1/60 s, in windowed and headless mode | ✅ (author) |
| T2 | Scheduling method | fixed timeline `t0 + n * period` | 💡 proposed |
| T3 | Falling behind | small catch-up, then re-sync | 💡 proposed |
| T4 | Wait precision (Windows timer, spin) | — | ❓ measure first |
| T5 | Game server tick | — | ❓ |
