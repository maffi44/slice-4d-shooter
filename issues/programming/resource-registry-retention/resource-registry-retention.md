# Bound the lifetime of runtime resource registry entries

- **Status:** Done
- **Priority:** High
- **Created:** 2026-11-24
- **Closed:** 2026-11-24
- **Resolution:** Done
- **Context:** [Reproduction and cleanup constraints](resource-registry-retention-context.md)

Fyrox releases unused resource handles after expiry and keeps their UUID/path mappings in memory. Repeated requests for the same path reuse its mapping. The author considers the observed growth with distinct paths immaterial for the current game.

[accepted] Keep Fyrox 1.0 and the existing [resource request and retention contract](../unified-resource-requests/). The engine provides an unscheduled `prune_resource_registry()` for a future workload that demonstrates excessive registry growth. An explicit call removes only mappings whose resources have already left the cache. The same path can acquire a new UUID after cleanup. The engine tick continues to perform only Fyrox's ordinary cache maintenance.
