# Preserve user settings when the FPS field is invalid

- **Status:** Open — not implemented
- **Priority:** Normal
- **Complexity:** Moderate
- **Created:** 2026-11-20
- **Stable address:** `issues/programming/preserve-invalid-user-settings/`
- **Context:** [Review evidence and reasoning](preserve-invalid-user-settings-context.md)

## Task

An invalid hand-edited render rate can erase unrelated preferences at startup. Preserve valid fields and distinguish an absent settings file from an existing one that could not be read or parsed.

## Possible solution [proposed]

Recover an invalid FPS field locally with a warning and a defined fallback. Distinguish a missing file, an unreadable file and invalid JSON. For an unrecoverable file, keep its contents through shutdown and choose an explicit write destination or disable automatic saves for that run. Preserve the executable-local lookup precedence.

## Completion criteria

Using disposable paths, check invalid FPS with volume `0.42`, a legacy file without FPS, an absent file and malformed JSON. Verify both the effective settings and on-disk contents after normal exit. Exercise both supported lookup locations.
