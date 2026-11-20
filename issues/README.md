# Issues

Tasks cover all project areas under `issues/<area>/<slug>/`; programming is currently the only area. The task lists are separate so that the growing history of closed tasks is not loaded during everyday work:

- **[Open tasks](open-tasks.md)**: current work, sorted by priority and status. Start here.
- **[Closed tasks](closed-tasks.md)**: completed, obsolete or otherwise closed tasks, newest first. Read it only when looking for history.

Full rules and examples: [AGENTS.md](../AGENTS.md#issues-and-decision-context).

## Task layout

- A task directory, e.g. `programming/global-allocator/`, is the task's permanent address. It is never renamed or moved, whatever the priority or status.
- The directory contains a concise main file `<slug>.md` and, when extended discussion is needed, `<slug>-context.md`. Filenames never change; status, priority and dates live in task metadata and in the lists.
- Links between tasks, documents and source comments point to the task directory, with the role/section described: “decision rationale in this task's context, §19”.

## Open task list order

Rows are sorted by priority (High → Normal → Long-term), then by status group, then by creation date (oldest first):

1. **Awaiting author review**: implemented, needs the author's acceptance or feedback.
2. **Decision or design pending**: needs discussion before or during implementation.
3. **Not implemented**: scope is clear enough to implement.
4. **Backlog**: deliberately deferred.

Priority and horizon are planning decisions, not claims of implementation.

## Closing a task

Only after the author confirms the closure: set **Status**, **Closed** and **Resolution** in the task's metadata, then move its row from the open list to the top of the closed list. Files and the directory stay in place. Resolutions:

- **Done**: completion criteria met and accepted.
- **Obsolete**: no longer relevant; state why.
- **Superseded**: replaced by another task; link it.
- **Rejected**: decided not to do; state why.
- **Duplicate**: link the task that remains.

Reopening moves the row back and records the reason in the task context.

## Review disposition — 2026-11-16

The author's response to the project review created 12 tasks (points 1–7, 9, 10, 13, 15, 16; see the “Review” column of the open list). High priority is exactly points 2, 3, 6 and 10; points 2 and 6 were particularly emphasized. GPU upload grouping and overflow protection (1 and 5) are long-term.

- Point 8 (standalone nested-world verification scenario): no issue requested now.
- Point 11 (performance instrumentation): deferred without a new issue.
- Point 12: keep the existing allocator task; tick scheduling also already has a separate task.
- Point 14 (test suite): no issue requested now. Specific verification criteria inside bug tasks are not a separate test-suite project.
- Points 17–19: documentation changes made directly (concise AGENTS, current task/context split, less categorical rules), not new backlog issues.

The engine refactoring was accepted by the author as complete on 2026-11-16; its review follow-ups remain open as separate tasks. Each new context records the review finding and the author's response; historical context is retained for the seven earlier tasks.

## Follow-up after the first refactoring commit

On 2026-11-16 the author confirmed initial refactoring commit `caea9ec`. Startup ownership/nonblocking loading was implemented in its own task (closed 2026-11-20). Borrowed broadcast delivery remains a separate, unimplemented task.

## Tracking convention history

- [superseded] 2026-11-16: status and priority encoded in filenames (`high-priority-` prefix; `done-YYYY-MM-DD-` at completion) with one index table. Rationale for stable directories: [refactoring context](programming/engine-refactoring-world-isolation/), “2026-11-16 — Review response and documentation migration”.
- [accepted] 2026-11-20: the author chose separate open and closed task lists and constant filenames. Stable directories are kept, so cross-references survive closure; prefixes were removed from existing files. Reasons: renaming files on every priority or status change was inconvenient, and a growing closed-task list should not take agent context during everyday work. A considered alternative, moving files between `open/` and `done/` folders, was rejected because every direct link would break on closure.
