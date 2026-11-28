# Clarify engine names and code layout

- **Status:** Open — design pending
- **Priority:** Normal
- **Complexity:** Moderate
- **Created:** 2026-11-28

## Scope and decisions

[accepted] Discuss representative engine functions with the author, document the agreed code style in `AGENTS.md`, then apply it across the engine in reviewable groups. The initial naming rule comes from the [dynamic texture review](../render-resources/).

[proposed] Use blank lines to separate logical steps within a function. Carry the domain term through related types, fields and locals where a reader would otherwise have to look up the surrounding type or function. The author's estimate was that roughly 30–50% of terse local names might benefit from another word. Choose each change for the meaning it adds.

## Completion criteria

- Agree on before-and-after examples of naming and logical spacing, including a case where the short name already conveys the meaning.
- Record the agreed guidance in `AGENTS.md` and apply it to the reviewed engine code. Preserve public data formats and behavior. Verify affected callers and binaries.
