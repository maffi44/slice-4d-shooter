# Separate development CI from release deployment

- **Status:** Open — not implemented
- **Priority:** Normal
- **Complexity:** Complex
- **Created:** 2026-11-16
- **Stable address:** `issues/programming/ci-validation-and-release/`
- **Context:** [ci-validation-and-release-context.md](ci-validation-and-release-context.md) — rationale, discussion and implementation history; read relevant sections before changing a prior decision.

## Task

Add non-publishing checks for `dev` and pull requests, covering relevant workspace builds and platform-specific paths. Separate validation from release/appcast/server deployment.

Inspect current independent Windows and server jobs, and build/deploy the exact validated commit instead of whatever `git pull origin main` returns later. Define compatible client/server artifact promotion, manual-dispatch behavior and failure handling.

Completion: routine checks cannot publish/restart production, failures block intended promotion, artifacts identify the tested revision, and README/workflow documentation matches behavior. Keep version bumps and production deployment explicitly authorized. Source: `.github/workflows/deploy.yml`.
