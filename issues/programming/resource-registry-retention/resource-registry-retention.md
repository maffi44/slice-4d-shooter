# Bound the lifetime of runtime resource registry entries

- **Status:** Open — design pending
- **Priority:** High
- **Complexity:** Moderate
- **Created:** 2026-11-24
- **Context:** [Reproduction and cleanup constraints](resource-registry-retention-context.md)

Fyrox retains the path and UUID for every requested map or sound until shutdown, including failed requests. Cache expiry releases resource data. In a long-lived process loading distinct external paths, the separate registry grows and a lookup by path scans its entries.

[open] Decide when the engine can forget a runtime UUID and how cleanup can follow eviction at low cost. Keep cache maintenance inside the resource system and preserve the [actor and preparation contract](../unified-resource-requests/).

Completion requires a repeatable churn check with successful and missing paths showing bounded registry size after expiry. Verify shared pending requests, session pins, failed loads, expiry followed by a concurrent request for the same path, and headless loading. Check the work performed on ticks with no evictions.
