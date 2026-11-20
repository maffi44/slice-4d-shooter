# Context for emit PlayerBeatLevel once per completed level

[Current task](single-level-completion-event.md)

## 2026-11-20 code review on `4007599`

After `level_transition_timer > 3.0`, `FinalTrgger::tick` broadcasts `PlayerBeatLevel` every tick. With a ready destination this already repeats until the five-second transition. While loading it continues, and after a destination error `transition_finished` stops transition attempts while the broadcast continues for the lifetime of the current world. The review found no current consumer of this event. Every broadcast still visits the actors.

This follows from the two independent timer conditions in the [final trigger](../../../src/actor/final_trigger.rs) and the broadcast loop in [Messenger](../../../src/engine/world/messenger.rs). The [asynchronous transition](../async-world-preparation/) deliberately leaves the current world running after a load failure.
