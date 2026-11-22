# Context for emit PlayerBeatLevel once per completed level

[Current task](single-level-completion-event.md)

## 2026-11-20 code review on `4007599`

After `level_transition_timer > 3.0`, `FinalTrgger::tick` broadcasts `PlayerBeatLevel` every tick. With a ready destination this already repeats until the five-second transition. While loading it continues, and after a destination error `transition_finished` stops transition attempts while the broadcast continues for the lifetime of the current world. The review found no current consumer of this event. Every broadcast still visits the actors.

This follows from the two independent timer conditions in the [final trigger](../../../src/actor/final_trigger.rs) and the broadcast loop in [Messenger](../../../src/engine/world/messenger.rs). The [asynchronous transition](../async-world-preparation/) deliberately leaves the current world running after a load failure.

## 2026-11-22 implementation

Two variants: a "completion sent" flag, or detecting the threshold crossing from the previous and current timer values. [accepted] Crossing detection: it needs no new state, and the timer only grows while the trigger is triggered, so the condition is true exactly once. [rejected] A flag: equivalent, but one more field to keep consistent.

The transition logic moved from `tick` into `advance_level_transition(delta, messenger, resources)`. Constructing `EngineSharedSystems` in a test requires net, UI and a runtime. The method needs only the messenger and the resource system. The 3 s and 5 s literals became `LEVEL_COMPLETION_TIME` and `LEVEL_TRANSITION_TIME`.

Tests use 0.25 s ticks. Ready destination (`capture_the_flag_map` loaded beforehand): 0 broadcasts at 2 s, then 1 broadcast and 1 `ReplaceRootWorld` after 10 s total. Delayed destination (a pending handle): after 8 s, 1 broadcast and no replacement. After `commit_ok` and 2 more seconds, still 1 broadcast and exactly 1 replacement. Missing map: 1 broadcast, no replacement, and `transition_finished` is set.

Verification on uncommitted changes over `69d51e4` (Linux x86_64): `cargo build --release --workspace --locked` and `cargo test --workspace --locked` pass without compiler warnings, and so does `cargo check --target x86_64-pc-windows-gnu --bins --locked`. With loopback matchmaking settings, the headless client (15 s) and the Vulkan client (20 s) on the default map reached `engine: ready` with no panics.

## 2026-11-22 review and closure [accepted]

Closed at the author’s request after review.
