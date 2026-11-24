# Runtime resource registry context

[Current task](resource-registry-retention.md)

## 2026-11-24 — Dependency update review

[accepted] An isolated probe used Fyrox 1.0.1, the game's `LoaderOwnedIo` and a registered WAV loader. After requesting 128 distinct missing WAVs, awaiting their errors and advancing the cache past 60 seconds, the manager contained zero resource objects and the registry still contained 128 UUID/path pairs. `ResourceRegistry::path_to_uuid` iterates over the map. Repeated visits to a fixed set of paths reuse entries.

`ResourceManagerState::update` owns expiry and only publishes the removed path. Its event broadcaster also sends strong handles on addition and successful load. A subscriber that keeps these events until a later tick extends resource lifetimes. A delayed removal by path could erase the mapping of a resource requested again in the meantime.

[open] Compare cleaning up under the manager lock at eviction with reconciling live entries after eviction. Account for header locks and allocations on the simulation thread. Establish whether callers may ever store a bare UUID across expiry. Preserve sharing while a handle is alive and keep registry writes off disk.
