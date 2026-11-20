# Define delivery of pending requests when replacing the root world

- **Status:** Open — design pending
- **Priority:** Normal
- **Complexity:** Moderate
- **Created:** 2026-11-20
- **Stable address:** `issues/programming/root-replacement-mailbox/`
- **Context:** [Review evidence and reasoning](root-replacement-mailbox-context.md)

## Task

Define what the engine owes the outgoing world when handling a replacement request also destroys the mailbox that supplied it.

## Possible solution [proposed]

Decide whether replacement cancels the outgoing world's remaining requests. To guarantee delivery, transfer its pending mailbox to the engine before replacing A, finish those requests in FIFO order and specify when to process requests from newly activated worlds. An explicit cancellation contract is also possible. Keep capacity reuse in the ordinary tick path.

## Completion criteria

Document the chosen policy in the architecture reference. Verify through the owner that replacement followed by quit, two replacements from A, and a request emitted by B during activation each have the documented effect.
