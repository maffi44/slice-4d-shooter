# Define and preserve message ordering

- **Status:** Done — reviewed by the author
- **Priority:** High
- **Created:** 2026-11-16
- **Closed:** 2026-11-20
- **Resolution:** Done
- **Stable address:** `issues/programming/message-ordering/`
- **Context:** [message-ordering-context.md](message-ordering-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Task

`NetSession` receives packets in arrival order, but `Messenger` delivers queued messages with `Vec::pop()`. Two score updates, 1 followed by 2 in the same tick, can be applied as 2 then 1, leaving an obsolete score. State assignment is order-dependent.

Define ordering for state updates, events and actor lifecycle commands; inspect interactions between broadcast, direct and command queues. Choose a FIFO or other explicit policy without repeated allocation or shifting all elements. Do not mechanically reverse every queue without considering spawn/remove behavior.

Completion: ordered updates preserve the newest intended state, lifecycle delivery semantics are documented and buffers retain reusable capacity. Source: `world/messenger.rs`, `actor/session_controller/net_session.rs`, `actor/session_controller.rs`.

Constraint from [main actor ID collisions](../main-actor-id-collisions/) (2026-11-20): when `ChangeMyId` displaces another actor, two `IWasChangedMyId` broadcasts are queued (old -> new, then new -> random). ID-tracking receivers such as `MoveWBonusSpot` apply them correctly only if the second is delivered first (current LIFO). A new policy must preserve this, e.g. by delivering ID changes atomically or with both IDs in one notification. [resolved] by reordering the two notifications (see Result).

## Result

- [accepted] One FIFO order for everything a world's `Messenger` queues: broadcast messages, direct messages and commands are handled in the order they were sent (author's choice of FIFO, 2026-11-20). Items sent during handling are appended after everything already queued (breadth-first). The outer-world mailbox is FIFO as well.
- [accepted] Storage: one `VecDeque<Queued>` (enum of broadcast, direct and command), so the pump reads a single sequential buffer (author's preference, 2026-11-20). `CommandType::SpawnActor` holds a `Box<ActorWrapper>`, which shrinks `Command` from 1520 to 48 bytes and keeps a queue slot at 256 bytes (`Message` 240 bytes + recipient id); the cost is one allocation per spawn. A unit test guards the slot size. The buffers keep their capacity; nothing is shifted. The earlier per-kind queues are superseded (context, 2026-11-20).
- [accepted] Lifecycle semantics follow the same order: a direct message sent after `SpawnActor` reaches the spawned actor; a message sent after `RemoveActor` is dropped; `RemoveActor` followed by `SpawnActor` with the same id leaves the new actor (previously the new actor was removed). Ids are resolved when an item is handled.
- [accepted] ID collisions: `Actors::displace_actor_at` moves the previous holder of a target id (and queues its `IWasChangedMyId`) before the changed actor notifies, so FIFO receivers apply new -> random before old -> new and never rename one reference twice.

Verification and the analysed behavior changes: implementation log in the context file.
