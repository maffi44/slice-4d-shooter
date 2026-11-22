# Context for skip sound source construction when playback is unavailable

[Current task](headless-sound-builder-allocation.md)

## 2026-11-20 code review on `4007599`

Both sound spawn entry points construct `SoundSourceBuilder` before checking the optional buffer. `spawn_source` then checks for a sound engine. Headless skips buffer acquisition and device creation, yet reaches the builder on a sound call. The locked `fyrox-sound` 0.36.2 implementation initializes the bus with `AudioBusGraph::PRIMARY_BUS.to_string()`, which allocates. This is source evidence for an avoidable allocation per call. Its impact on total tick time has not been measured.

The no-device case can also occur in a graphical run when opening the sound engine fails. A present handle with a pending decode is a separate, supported case. See [audio entry points](../../../src/engine/shared_systems/audio.rs), [resource requests](../../../src/engine/shared_systems/resources.rs) and the [resource system contract](../resource-system/).

## 2026-11-22 implementation

Order of checks: buffer (`sound_buffer.as_ref()?` before `spawn_source`), then sound engine, then the builder closure. Variants: [accepted] a builder closure inside `spawn_source`, so one helper holds the check and both entry points can't diverge. The test can also observe deterministically that the closure did not run. [rejected] Duplicating an `if self.sound_engine.is_none()` check in each entry point: two places to keep in sync, and the property can only be tested indirectly.

`master_volume` is copied into a local before the call because the closure is evaluated while `self` is mutably borrowed.

Test: a headless `AudioSystem` with a real decoded-buffer handle returns `None` and the builder flag stays false. Absent buffers return `None` for both entry points, and no pending buffer is recorded. Graphical playback is covered only by the unchanged code path and the windowed run reaching `engine: ready`. Hearing the sounds needs a manual session.

Verification on uncommitted changes over `69d51e4` (Linux x86_64): `cargo build --release --workspace --locked` and `cargo test --workspace --locked` pass without compiler warnings, and so does `cargo check --target x86_64-pc-windows-gnu --bins --locked`. With loopback matchmaking settings, the headless client (15 s) and the Vulkan client (20 s) on the default map reached `engine: ready` with no panics.

## 2026-11-22 review and closure [accepted]

Closed at the author’s request after review.
