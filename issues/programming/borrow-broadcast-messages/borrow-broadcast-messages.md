# Deliver broadcast messages by reference

- **Status:** Open — not implemented
- **Priority:** Normal
- **Created:** 2026-11-16
- **Stable address:** `issues/programming/borrow-broadcast-messages/`
- **Context:** [borrow-broadcast-messages-context.md](borrow-broadcast-messages-context.md)

## Task

Pass a borrowed message to broadcast recipients instead of cloning the entire message for every actor. The message remains owned by Messenger/the local dispatch variable for the duration of delivery.

Current path: `Messenger::process_messages` pops a broadcast message and calls `actor.recieve_message(message.clone(), ...)` for each receiver. Inspect `Actor::recieve_message`, the `actor_wrapper!` forwarding implementation and all handlers.

Prefer a simple borrowed callback (`&Message`) where practical; direct delivery can also borrow its popped message if a common signature keeps the interface simple. Handlers that must retain data or enqueue an owned payload copy/clone only that payload explicitly. Do not store references beyond the callback or replace copies with unnecessary per-message Arc allocations.

Preserve sender filtering, remote-sender metadata and current delivery semantics. Coordinate [message ordering](../message-ordering/) without silently changing order in this small task. Keep the intentional spelling `recieve_message`.

Completion: no unconditional full-message clone per broadcast recipient, all actors/bins compile, and handlers still produce the same intended effects. Verify with relevant workspace checks. This is expected to be a small mechanical change; implementation has not been requested in this turn.
