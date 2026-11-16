# Close connections when their owner is removed

- **Status:** Open — not implemented
- **Priority:** High
- **Created:** 2026-11-16
- **Stable address:** `issues/programming/session-connection-lifetime/`
- **Context:** [high-priority-session-connection-lifetime-context.md](high-priority-session-connection-lifetime-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Task

Removing a `SessionController` or its world drops plain connection handles but leaves transports in shared `NetSystem`. Incoming packets can accumulate without a consumer.

Tie connection lifetime to its logical owner. Cover actor removal, root replacement, nested-world destruction, failed connection attempts and application shutdown. Choose the simplest ownership/cleanup mechanism after examining all paths; a bare `Drop` on a copyable ID cannot access shared transport by itself.

Completion means removed owners leave no active connection or orphan receive queue/task, and surviving owners retain their connections. Verify repeatedly, including while connecting. Source: `src/actor/session_controller/net_session.rs`, `src/engine/shared_systems/net.rs`, world/actor removal paths.
