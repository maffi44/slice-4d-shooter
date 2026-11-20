# Context for resolve the root world's WorldFinished behavior

[Current task](root-world-finished.md)

## 2026-11-20 code review on `4007599`

Commit `858c5ae` changed the `WorldFinished` arm in `Engine::handle_outer_world_message` from setting `quit_requested` to doing nothing. The architecture reference still promises application exit at the root. The enum comment agrees with the current no-op. The review found no current gameplay sender, so this discrepancy affects the caller contract without evidence of a broken shipped level.

`QuitApplication` already carries an explicit exit request. For a nested world the owner actor decides how to handle completion. Neither nested ownership nor the resource integration establishes which behavior a root-world caller should expect. See [root handler](../../../src/engine.rs), [message declaration](../../../src/engine/world.rs), [architecture](../../../docs/engine-architecture.md) and [world preparation](../async-world-preparation/).
