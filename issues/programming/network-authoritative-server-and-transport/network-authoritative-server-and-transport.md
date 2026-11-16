# Authoritative server and transport

- **Status:** Backlog — design pending
- **Priority:** Normal
- **Created:** 2026-11-12
- **Stable address:** `issues/programming/network-authoritative-server-and-transport/`
- **Context:** [network-authoritative-server-and-transport-context.md](network-authoritative-server-and-transport-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Goal

Move authoritative gameplay state to the server and replace emulated peer-to-peer relaying. Evaluate transport separately from authority: changing WebRTC alone does not make the game authoritative.

## Current state and scope

`NetSystem` is already transport-only; game-session logic lives in `SessionController`. The existing server still relays client-authoritative gameplay messages. Earlier uncertainty about adopting this client split is historical, not the present implementation.

Design server simulation/tick, prediction, reconciliation, lag compensation, protocol and compatible deployment before implementation. Decide whether to retain WebRTC or adopt another transport. Context records why WebRTC was chosen and why the direction changed.

Immediate [connection lifetime](../session-connection-lifetime/) and ordinary-priority [TCP framing](../server-tcp-framing/) fixes are separate and need not wait for this rewrite.
