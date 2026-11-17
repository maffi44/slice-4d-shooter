# Issues

Start with the main task; consult its sibling `-context.md` when the rationale, earlier discussion or implementation history is needed. Each link below is a permanent task directory, independent of filename priority/status changes. Rules and examples: [AGENTS.md](../AGENTS.md#issues-and-decision-context).

Open filenames have no date. High priority prefixes both files with `high-priority-`. After the author's confirmation, both receive `done-YYYY-MM-DD-` using the completion date; their directory stays unchanged. Priority and horizon are planning decisions, not claims of implementation.

## Programming

| Task | Priority | Status | Review point |
|---|---|---|---|
| [Prepare and validate worlds asynchronously](programming/async-world-preparation/) | High | Open — not implemented | 10 |
| [Preserve main_actor_id during actor ID collisions](programming/main-actor-id-collisions/) | High | Open — not implemented | 6 |
| [Define and preserve message ordering](programming/message-ordering/) | High | Open — not implemented | 3 |
| [Close connections when their owner is removed](programming/session-connection-lifetime/) | High | Open — not implemented | 2 |
| [Separate development CI from release deployment](programming/ci-validation-and-release/) | Normal | Open — not implemented | 13 |
| [Clean dependencies and obsolete build configuration](programming/dependency-and-build-cleanup/) | Normal | Open — not implemented | 16 |
| [Engine refactoring: isolated, nestable worlds](programming/engine-refactoring-world-isolation/) | Normal | ✅ Done — accepted 2026-11-16 | — |
| [Engine tick scheduling](programming/engine-tick-rate/) | Normal | Open — decision or implementation pending | — |
| [Global allocator: keep, replace or remove?](programming/global-allocator/) | Normal | Open — decision or implementation pending | 12 |
| [In-world 4D screens](programming/in-world-4d-screens/) | Normal | Backlog — design pending | — |
| [Preserve short presses and multiple input bindings](programming/input-button-state/) | Normal | Open — not implemented | 4 |
| [Split mixed responsibilities into logical submodules](programming/logical-module-splitting/) | Normal | Open — not implemented | 15 |
| [Maps out of sync with the parser](programming/maps-out-of-sync-with-parser/) | Normal | Open — decision or implementation pending | — |
| [Authoritative server and transport](programming/network-authoritative-server-and-transport/) | Normal | Backlog — design pending | — |
| [Responsive engine startup and MainLoop ownership](programming/nonblocking-engine-startup/) | Normal | First implementation — awaiting author review | — |
| [OpenGL context contention during rendering](programming/opengl-context-contention/) | Normal | OpenGL removal implemented — awaiting author acceptance | — |
| [Deliver broadcast messages by reference](programming/borrow-broadcast-messages/) | Normal | Open — not implemented | — |
| [Resource System](programming/resource-system/) | Normal | Backlog — design pending | — |
| [Frame server TCP messages and bound reads](programming/server-tcp-framing/) | Normal | Open — not implemented | 7 |
| [Separate real elapsed time and simulation time if needed](programming/world-time-semantics/) | Normal | Open — not implemented | 9 |
| [Protect GPU upload buffers from overflow](programming/gpu-buffer-overflow/) | Long-term | Backlog — long-term | 5 |
| [GPU upload grouping and coherent data](programming/gpu-data-grouping/) | Long-term | Backlog — long-term | 1 |

## Review disposition — 2026-11-16

The author's response to the project review created 12 tasks (points 1–7, 9, 10, 13, 15, 16). High priority is exactly points 2, 3, 6 and 10; points 2 and 6 were particularly emphasized. GPU upload grouping and overflow protection (1 and 5) are long-term.

- Point 8 (standalone nested-world verification scenario): no issue requested now.
- Point 11 (performance instrumentation): deferred without a new issue.
- Point 12: keep the existing allocator task; tick scheduling also already has a separate task.
- Point 14 (test suite): no issue requested now. Specific verification criteria inside bug tasks are not a separate test-suite project.
- Points 17–19: documentation changes made directly (concise AGENTS, current task/context split, less categorical rules), not new backlog issues.

The engine refactoring was accepted by the author as complete on 2026-11-16 (renamed to `done-2026-11-16-…`); its review follow-ups remain open as separate tasks. Each new context records the review finding and the author's response; historical context is retained for the seven earlier tasks.

## Follow-up after the first refactoring commit

On 2026-11-16 the author confirmed initial refactoring commit `caea9ec`. Startup ownership/nonblocking loading has a first separate-loader implementation awaiting author review in its own task. Borrowed broadcast delivery remains a separate, unimplemented task.
