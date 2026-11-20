# Review the engine for cache-friendly memory layout

- **Status:** Open — research pending
- **Priority:** Long-term
- **Created:** 2026-11-20
- **Stable address:** `issues/programming/cache-friendly-memory-layout/`

## Question

Where does the engine's data layout work against the CPU cache, and which of those places actually matter? This is a review and research task, not a predetermined optimization: the expected answer may be that a given place is fine.

Motivating example: [message ordering](../message-ordering/) (2026-11-20). Three per-kind queues plus an order queue were merged into one `VecDeque`, and `CommandType::SpawnActor` was boxed, shrinking `Command` from 1520 to 48 bytes and keeping a queue slot at 256 bytes. The measurements and the reasoning (including the caveat that a broadcast is dominated by iterating all actors) are in that task's context file.

## What to review [open]

- Sizes of hot types and enums dominated by one large variant: `ActorWrapper` (1504 bytes, as large as `PlayerFor2d3dExample`), `Message` (240 bytes, dominated by `SessionControllerMessage::SendToPlayer`), physics colliders, render data.
- Iteration patterns in tick/frame paths: `Actors` as `HashMap<ActorID, ActorWrapper>` (broadcast delivery visits every actor and its whole slot), physics passes over colliders, render-data collection and GPU upload buffers.
- Pointer chasing and scattered small allocations (`Vec`/`Box` per actor or per shape), hot and cold fields mixed in one struct, data copied between buffers each tick.
- Whether a change is worth it: measure (e.g. `perf stat` cache misses, frame/tick time) before proposing complex layouts; keep the simplest design where the cost is negligible (AGENTS.md working principles).

## Expected result

A list of findings with evidence (type sizes, profiles or reasoning marked as inference), each classified as: no problem, minor, worth a separate task. Concrete changes go into separate tasks or are agreed with the author before implementation.
