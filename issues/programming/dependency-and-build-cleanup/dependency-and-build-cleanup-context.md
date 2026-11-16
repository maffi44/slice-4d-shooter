# Context: Clean dependencies and obsolete build configuration

[Current task](dependency-and-build-cleanup.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

Review point 16. Author requested an issue (2026-11-16). Browser support was removed in f3e2128, but browser-related dependencies and the commented wasm target remain. Their actual use must be checked before removal; a name alone is not evidence that a dependency is unused.

The root Cargo.lock already fixes resolved versions even where manifests use `*`; the issue is deliberate compatibility bounds and maintenance, not claiming current builds resolve arbitrary versions. Coordinate [CI work](../ci-validation-and-release/) and keep [allocator](../global-allocator/) decisions separate.
