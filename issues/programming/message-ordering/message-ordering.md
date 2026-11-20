# Define and preserve message ordering

- **Status:** Open — not implemented
- **Priority:** High
- **Created:** 2026-11-16
- **Stable address:** `issues/programming/message-ordering/`
- **Context:** [message-ordering-context.md](message-ordering-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Task

`NetSession` receives packets in arrival order, but `Messenger` delivers queued messages with `Vec::pop()`. Two score updates, 1 followed by 2 in the same tick, can be applied as 2 then 1, leaving an obsolete score. State assignment is order-dependent.

Define ordering for state updates, events and actor lifecycle commands; inspect interactions between broadcast, direct and command queues. Choose a FIFO or other explicit policy without repeated allocation or shifting all elements. Do not mechanically reverse every queue without considering spawn/remove behavior.

Completion: ordered updates preserve the newest intended state, lifecycle delivery semantics are documented and buffers retain reusable capacity. Source: `world/messenger.rs`, `actor/session_controller/net_session.rs`, `actor/session_controller.rs`.

Constraint from [main actor ID collisions](../main-actor-id-collisions/) (2026-11-20): when `ChangeMyId` displaces another actor, two `IWasChangedMyId` broadcasts are queued (old -> new, then new -> random). ID-tracking receivers such as `MoveWBonusSpot` apply them correctly only if the second is delivered first (current LIFO). A new policy must preserve this, e.g. by delivering ID changes atomically or with both IDs in one notification.
