# Discover bundled destination maps from level JSON

- **Status:** Open — design pending
- **Priority:** Normal
- **Complexity:** Moderate
- **Created:** 2026-11-24

## Problem

The generated [embedded asset catalog](../embedded-asset-catalog/) scans Rust markers. A final trigger declares its destination in the map's JSON. Earlier releases repeated declarations for linked destinations. Since 2026-11-25 the client ships only CTF, with no linked destination. The discovery use case returns when linked levels ship again.

## Scope and decisions

[accepted] Track build-time discovery of destination maps from level descriptions. Keep the existing declarations until this task is implemented.

[proposed] Start from explicit bundled entry maps, follow `final_trigger.next_level_name` recursively, and deduplicate by logical map key. Preserve standalone maps that a player can select by CLI. Runtime disk overrides retain the [resource system lookup order](../unified-resource-requests/).

[open] Choose the entry map declarations and define the diagnostic or declaration for a reference to a disk-only destination. Packaged legacy maps can contain older gameplay schemas, so discovery must account for the actual map descriptions that currently ship.

## Completion criteria

- A bundled root that leads to B and then C includes both destinations without separate Rust markers. Shared destinations, self-transitions and cycles terminate with one catalog entry per key.
- Changing or removing a JSON reference updates the generated catalog on the next build. Verify directory-qualified names, explicit standalone entry maps and the agreed handling of missing destinations.
- Preserve the intended shipped map set and document how an author adds an entry map or a linked level.
