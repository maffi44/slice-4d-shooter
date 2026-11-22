# Context for resolve the root world's WorldFinished behavior

[Current task](root-world-finished.md)

## 2026-11-20 code review on `4007599`

Commit `858c5ae` changed the `WorldFinished` arm in `Engine::handle_outer_world_message` from setting `quit_requested` to doing nothing. The architecture reference still promises application exit at the root. The enum comment agrees with the current no-op. The review found no current gameplay sender, so this discrepancy affects the caller contract without evidence of a broken shipped level.

`QuitApplication` already carries an explicit exit request. For a nested world the owner actor decides how to handle completion. Neither nested ownership nor the resource integration establishes which behavior a root-world caller should expect. See [root handler](../../../src/engine.rs), [message declaration](../../../src/engine/world.rs), [architecture](../../../docs/engine-architecture.md) and [world preparation](../async-world-preparation/).

## 2026-11-22 decision and implementation

Options presented: (A) restore the root exit, (B) exit only on `QuitApplication` and fix the documentation. Recommended B: exit stays an explicit request, `WorldFinished` has no current sender, so play is unchanged. The author chose B.

The architecture reference previously said that the root exits and called the enum comment stale. Both now say that the root ignores `WorldFinished`. The test builds a real headless `Engine` through `engine::load` and calls the handler directly. It does not tick, so no actor opens a network connection: the session controller connects only from its tick.

Verification on uncommitted changes over `69d51e4` (Linux x86_64): `cargo build --release --workspace --locked`, `cargo test --workspace --locked` (23 client tests including the new one) and `cargo check --target x86_64-pc-windows-gnu --bins --locked` pass without compiler warnings. No separate headless or windowed run was made for this change: no actor sends `WorldFinished`, so the change only touches comments, documentation and the test. The headless engine in the test covers the root path. The headless and Vulkan runs from the same session, recorded in the other tasks, reached `engine: ready`.

## 2026-11-22 review and closure [accepted]

The original test used the production settings loader. With an invalid FPS field it could overwrite a developer's other settings with defaults. The headless fixture now prepares the map with in-memory settings and no persistence writer. It exercises the root handler without accessing user configuration. Production settings recovery remains a [separate task](../preserve-invalid-user-settings/).

Verification on the working tree over `69d51e4`: the Linux release workspace build, workspace tests and Windows GNU bins check passed. The no-updater client reached `engine: ready` in both headless and Vulkan with loopback matchmaking settings. Headless logged a gamepad hotplug monitor error. On Linux the root handler test also passed twice with an isolated platform config directory. An invalid FPS and a custom volume survived byte-for-byte. Starting with no settings file created none.

Closed at the author’s request after review.
