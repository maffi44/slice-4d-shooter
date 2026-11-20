# Context for define delivery of pending requests when replacing the root world

[Current task](root-replacement-mailbox.md)

## 2026-11-20 code review on `4007599`

Trace an A mailbox containing `ReplaceRootWorld(B)` followed by `QuitApplication`. `Engine::tick` pops the replacement from A. `set_root_world` installs B and drops A. The next iteration polls B, so the quit request disappears. With two replacements from A only the first runs. This follows from the code. The existing FIFO tests pop items from the queue directly.

The FIFO guarantee describes the order of items that are delivered. It leaves ownership of pending outer requests across replacement unspecified. The same boundary matters if B enqueues a request during activation. See [engine tick and replacement](../../../src/engine.rs), [mailbox operations](../../../src/engine/world/messenger.rs) and [message ordering](../message-ordering/).
