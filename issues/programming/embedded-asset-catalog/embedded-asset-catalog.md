# Generate the embedded asset catalog from Rust markers

- **Status:** Done
- **Priority:** Normal
- **Created:** 2026-11-23
- **Closed:** 2026-11-23
- **Resolution:** Done — accepted by the author
- **Follow-up:** [Unified resource requests](../unified-resource-requests/) covers caller API, source resolution and startup.

In the Linux release client, the complete WAVs for shotgun fire (177,378 bytes) and flag capture (175,496 bytes) each occurred twice in read-only data. Each had two request sites. Runtime cache sharing did not affect binary duplication.

[accepted] 2026-11-23. Keep direct literal markers at the request site. Generate one inclusion per distinct marked path at build time. Scanning asset directories would also ship unreferenced files. Scanning Rust token trees handles comments, strings and nested groups without implementing a Rust parser. All client Rust sources contribute, including conditional modules. This replaces runtime registration of embedded gameplay WAVs from the [resource system](../resource-system/). World startup and future resource API changes are separate design topics.

Completion criteria: repeated markers share a single catalog entry, unmarked files do not enter the catalog, and existing requests retain their headless, preparation and cache behavior.

Verified on 2026-11-23 on Linux, uncommitted tree based on `7b48157`. The generated catalog contains 26 keys. Every marked WAV occurs byte-for-byte once in each of the three release clients. Isolated source fixtures covered duplicate markers across files, qualified nested invocations, raw string paths, ignored comments and string examples, an unmarked asset, removal of the last use and rejecting a computed argument. Workspace release build, Linux and Windows GNU bins checks and workspace tests passed (Matchbox tests used local sockets). Both headless and Vulkan reached engine ready with loopback settings; Vulkan loaded 22 embedded WAVs and headless loaded zero.
