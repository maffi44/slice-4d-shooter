# Context: Separate development CI from release deployment

[Current task](ci-validation-and-release.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

Review point 13. Author requested this task (2026-11-16), without high priority. Current workflow deploys on pushes to main and supports manual dispatch; server deployment need not wait for the Windows build, and remote pulling can select a different revision.

This issue does not authorize deployment now or creation of a separate test-suite backlog item (review point 14 was deferred). Existing checks may be wired into CI as appropriate. Context for compatibility: [authoritative server](../network-authoritative-server-and-transport/).
