# Clean dependencies and obsolete build configuration

- **Status:** Done — accepted by the author
- **Priority:** Normal
- **Created:** 2026-11-16
- **Closed:** 2026-11-24
- **Resolution:** Done
- **Stable address:** `issues/programming/dependency-and-build-cleanup/`
- **Context:** [Audit, prior version updates, decisions and verification](dependency-and-build-cleanup-context.md).

## Scope and result

[accepted] Remove unused direct dependencies and browser-only configuration from the native applications. On 2026-11-24 the earlier version update replaced wildcard requirements with compatible ranges. The subsequent native cleanup kept all surviving third-party lock versions and the product version. See the context for the per-dependency rationale.

The loader uses the engine's Tokio runtime on its existing OS thread. Native clocks use `std::time::Instant`. The client enables PNG decoding and native Vulkan, DX12 and Metal. Server-only shuffle and WebSocket helper imports no longer require Fyrox. The gameplay random paths and their seed semantics are preserved. [Shared network protocol](../shared-network-protocol/) tracks isolation of the application dependency trees. The allocator's resize defect remains in [its own task](../global-allocator/).

[accepted] Retain the configured Linux target and the target-qualified output paths used by the documented hosting and release commands. WinSparkle uses a Windows-only runtime dependency and the existing build dependency for the icon.

## Completion and verification

The Linux release workspace build, isolated app checks and Windows GNU workspace bins check passed. The context records before/after trees and binary sizes, image decoding, headless startup, offscreen Vulkan and the Windows resource checks. Windowed startup, a live network session and the macOS client build still need an environment with the relevant display, socket access and Apple dependencies.
