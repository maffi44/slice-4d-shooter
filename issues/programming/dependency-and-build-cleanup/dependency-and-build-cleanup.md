# Clean dependencies and obsolete build configuration

- **Status:** Open — not implemented
- **Priority:** Normal
- **Complexity:** Moderate
- **Created:** 2026-11-16
- **Stable address:** `issues/programming/dependency-and-build-cleanup/`
- **Context:** [dependency-and-build-cleanup-context.md](dependency-and-build-cleanup-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Task

Audit remaining web-only dependencies/configuration after wasm removal, unused direct dependencies, platform-specific dependencies and `*` requirements. Remove only verified leftovers; preserve deliberate capabilities.

Consider defaulting to the host target instead of forcing Linux globally; update documented paths and CI explicitly if changed. Pin compatible version ranges without casually updating Cargo.lock. Check build-script output handling and all client entry points.

Completion: each removed dependency/feature has an explained reason; Linux and relevant Windows/macOS build assumptions are checked, README and CI paths agree, and root product version/licenses remain unchanged. Dependency updates are targeted and require their own authorization.
