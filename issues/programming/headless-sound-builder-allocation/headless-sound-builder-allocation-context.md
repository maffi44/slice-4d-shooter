# Context for skip sound source construction when playback is unavailable

[Current task](headless-sound-builder-allocation.md)

## 2026-11-20 code review on `4007599`

Both sound spawn entry points construct `SoundSourceBuilder` before checking the optional buffer. `spawn_source` then checks for a sound engine. Headless skips buffer acquisition and device creation, yet reaches the builder on a sound call. The locked `fyrox-sound` 0.36.2 implementation initializes the bus with `AudioBusGraph::PRIMARY_BUS.to_string()`, which allocates. This is source evidence for an avoidable allocation per call. Its impact on total tick time has not been measured.

The no-device case can also occur in a graphical run when opening the sound engine fails. A present handle with a pending decode is a separate, supported case. See [audio entry points](../../../src/engine/shared_systems/audio.rs), [resource requests](../../../src/engine/shared_systems/resources.rs) and the [resource system contract](../resource-system/).
