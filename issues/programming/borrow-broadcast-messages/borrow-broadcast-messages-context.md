# Context: Deliver broadcast messages by reference

[Current task](borrow-broadcast-messages.md). Proposals below are not approved implementation decisions.

## 2026-11-16 — Author request

After accepting initial refactoring commit caea9ec, the author separately requested a small issue to avoid forgetting borrowed broadcast delivery. No elevated priority was specified.

[Refactoring context](../engine-refactoring-world-isolation/), §24.9 round 7 point 1, previously accepted broadcast clones because the message types were considered heap-free copies. Avoiding an allocation does not imply the whole-message copy is necessary. The new request targets those copies and API ownership; this is not a claim of measured performance gain.

The popped message is outside the Messenger buffers, so its immutable borrow can normally coexist with callbacks mutating Messenger to enqueue other work. Inspect handlers that currently consume message payloads and make only genuinely needed ownership transfers explicit. Queue ordering is a separate high-priority issue.
