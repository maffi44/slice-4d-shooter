# Context: Clean dependencies and obsolete build configuration

[Current task](dependency-and-build-cleanup.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

Review point 16. Author requested an issue (2026-11-16). Browser support was removed in f3e2128, but browser-related dependencies and the commented wasm target remain. Their actual use must be checked before removal; a name alone is not evidence that a dependency is unused.

The root Cargo.lock already fixes resolved versions even where manifests use `*`; the issue is deliberate compatibility bounds and maintenance, not claiming current builds resolve arbitrary versions. Coordinate [CI work](../ci-validation-and-release/) and keep [allocator](../global-allocator/) decisions separate.

## 2026-11-24 — full dependency update at the author's request

The author asked to update all crates to their latest versions and resolve any conflicts. This explicit request overrides the targeted-update rule for this one step. The root product version and the server package versions are unchanged.

[accepted] Every `*` requirement in the root, `game_server` and `matchmaking_server` manifests now names the latest major (or 0.x minor) version. A `cargo update --dry-run` with `*` left in place resolved `glam` to 19 versions from 0.14 to 0.34 and `bincode` to 3.0.0. `*` therefore does not reliably select a usable version, and explicit ranges also cover this task's pinning item. `matchbox_signaling_modified` already used caret ranges. Its manifest is unchanged; its lock entries were updated.

Major updates and required code changes:

- `wgpu` 27 → 30. Pipeline layouts take `Option<&BindGroupLayout>` and `immediate_size`. Vertex buffers are `Option`. `multiview` is now `multiview_mask`, and render passes also need it. Mipmap filters use `MipmapFilterMode`. The instance descriptor is passed by value and has `display: None`, because only the disabled GL backend uses it. `apply_limit_buckets: false` and `SurfaceColorSpace::Auto` keep the previous behavior. `get_current_texture` returns `CurrentSurfaceTexture`. Lost/Outdated reconfigures the surface as before. Occluded now skips the frame. Timeout and Validation are logged. The former `OutOfMemory` panic has no counterpart in this API. Presentation is `queue.present(texture)`.
- `fyrox-core`/`fyrox-resource`/`fyrox-sound` 0.36.2 → 1.0.1. These were pinned with `=0.36.2`, and the [Fyrox UI](../fyrox-ui/) evidence was written against 0.36.2. The 1.0 manager keeps a UUID registry. `request` registers new paths and `ResourceRegistryRefMut::drop` saves the registry synchronously to `data/resources.registry` through the manager's IO. Spawning a loading task requires a loaded registry. With `FsResourceIo`, the first request for each path would write that file from the tick thread into the player's working directory. [accepted] `LoaderOwnedIo` does not read or write. Its default `canonicalize_path` keeps logical keys and avoids the `unwrap` panic in `request` for paths that are not real files. `ResourceSystem::new` marks the registry loaded without scanning. Our loaders already ignored the IO argument. Other adaptations: `ResourceState` gained `Unloaded`, which reads as pending, and struct variants. `try_read` uses `data_ref_of_type`. `ResourceData::try_clone_box` returns `None`, and `#[reflect(non_cloneable)]` matches Fyrox's `SoundBuffer`. Loaders are registered through `loaders.lock()`. `try_get_source_mut` returns `Result`.
- `getrandom` 0.2 → 0.4: the `js` feature, a wasm leftover, was dropped; `getrandom::getrandom` is now `getrandom::fill`.
- `rand` 0.9 → 0.10: the extension trait `Rng` is now `RngExt`.
- `tokio-tungstenite` 0.28 → 0.30, `glam` 0.30 → 0.34, `blink-alloc` 0.3 → 0.4, `pollster` 0.4 → 1.0, `opener` 0.8 → 0.9, `dirs` 6 → 7, `libloading` 0.8 → 0.9, and build-dependency `syn` 2 → 3: no code changes were needed.
- `bincode`: 3.0.0 consists only of `compile_error!("https://xkcd.com/2347/")`, so the requirement is set to the latest working `2`. No code in the workspace uses bincode. It remains a removal candidate for this task. Fyrox 1.0 still depends on bincode 1.3 internally.
- `winit` stays on 0.30 (0.31 is still a beta).

Verification was run on the uncommitted Linux x86_64 tree based on `87ea043`. `cargo check --workspace --bins --tests`, the release workspace build, workspace tests (all passed) and the Windows GNU bins check completed without compiler warnings. The headless no-updater client ran for 15 s. The Vulkan client on an RTX 3070 Laptop GPU ran for 40 s on the default map. Both reached `engine: ready` with no errors, and neither created `data/resources.registry`. A window capture showed the world, skybox, HUD and UI textures. No pixel comparison with the previous build was made. The runs used the embedded connection settings but stayed offline, because online play starts only on a key press. Audible playback and online play were not checked.

## 2026-11-24 — Review follow-up

[accepted] A simulation tick queues buffer writes even when Metal reports an occluded window. With wgpu 30.0.2 on Vulkan llvmpipe, 1,024 writes of 4 KiB followed by polling increased live staging allocations by 1,024 and bytes by 4 MiB. An empty queue submission followed by completion released them. The scheduler now flushes these uploads on eligible skipped frames at intervals of at least 100 ms (about six nominal simulation ticks). The GPU idle check still applies. Successful acquisition immediately uses the ordinary frame submission. The flush interval limits work while hidden without postponing presentation when the window returns.

[superseded] The initial adaptation treated `Unloaded` as pending. Fyrox 1.0.1 returns an error when that handle is awaited because no load is in progress. Tick inspection now reports a terminal error as well. The regression case constructs the handle from a UUID and inspects it before any request.

[accepted] The [allocator](../global-allocator/) pin has been restored. The specific risks and remaining design decisions are recorded at High priority in that task, [lost surface recovery](../lost-render-surface/) and [runtime registry retention](../resource-registry-retention/).

### Verification after the review fixes

Linux x86_64, uncommitted tree based on `87ea043`. The locked release workspace build, Linux workspace bins/tests check and Windows GNU bins check passed without compiler warnings. Each client entry point passed 29 tests, including inspection of an unloaded resource. The sandbox denied local socket creation in the nine transport tests per client and 24 Matchbox tests.

The no-updater release client used loopback connection settings and isolated preferences. Headless checks passed for the bundled map, a local override A → external B → A, an external self-transition and an invalid local override with a path-specific diagnostic. The windowed run stopped at winit's `XOpenDisplayFailed`.

A separate Vulkan llvmpipe probe exercised the idle-queue check and 100 ms upload flush interval without a surface. Across 150 writes of 4 KiB, it submitted 22 batches, measured at most seven extra live allocations and returned to baseline after the final submission completed. Native window occlusion and restoration still require a graphical session.

## 2026-11-24 — Final allocator version

[accepted] The author requested the latest published allocator after the review. This supersedes the interim 0.3.1 pin above. Both manifests request the 0.4 series. The lock selects `blink-alloc` 0.4.0 and its `allocator-api2` 0.4.0 dependency. Other callers still use `allocator-api2` 0.2.21. See the [allocator investigation](../global-allocator/) for the two verified resize directions and the open High-priority decision.

On the uncommitted Linux x86_64 tree based on `87ea043`, the offline targeted update encountered an unrelated wasm resolution conflict. The allocator entries were resolved in an isolated Cargo lock and incorporated into the workspace lock. The locked dependency tree confirmed 0.4.0 for both consumers. Linux workspace bins/tests checking, the release workspace build and the Windows GNU bins check passed without compiler warnings. Each client test binary passed 29 tests. Local socket restrictions affected nine transport tests per client and 24 Matchbox tests.

With the final lock, the release client using isolated loopback settings again passed bundled startup, local override A → external B → A, an external self-transition and the malformed-map diagnostic in headless mode. The graphical attempt stopped at `XOpenDisplayFailed`.

## 2026-11-24 — Native binary dependency audit before implementation

First pass on clean source revision `baa9800`. Compared application manifests with Rust call sites, build.rs, embedded image sources and the locked crate sources. Inspected normal dependency trees for each application separately on Linux x86_64 and Windows GNU. For example:

```sh
cargo tree --offline --locked -p slice-4d-shooter --target x86_64-unknown-linux-gnu -e normal -i axum
cargo tree --offline --locked -p game_server --target x86_64-pc-windows-gnu -e normal
cargo tree --offline --locked -p matchmaking_server --target x86_64-unknown-linux-gnu -e normal
```

All three client bins share package dependencies. The Linux client, game server and matchmaking graphs contain 513, 295 and 114 distinct name/version pairs respectively, including the selected root. Windows GNU gives 488, 295 and 113. These counts describe Cargo's normal edges before cleanup. Build and dev edges are excluded. Linked executable sizes and build-time savings were not measured. The macOS offline tree lacked cached Apple crates (`block2` and `objc2-core-foundation`). This pass made no code or manifest changes.

### Direct removals and native substitutions

| Proposed edit | Evidence and constraint |
|---|---|
| Remove client `console_error_panic_hook` and `console_log` | No initialization or uses in any entry point. Native logging uses `env_logger`. Linux reverse trees lead from `web-sys` through `console_log` and from `wasm-bindgen` through the browser console crates. |
| Replace `web_time::Instant` with `std::time::Instant` | Five imports cover world/shared timing, the main loop, render scheduler and network session. Locked web-time 1.1.0 re-exports the standard library clock on native targets. |
| Replace the single `pollster::block_on` with `shared.runtime.block_on` | In `engine/loader.rs`, the named OS thread already owns the multi-thread Tokio runtime and uses it immediately before and after GPU initialization. Tokio polls the top-level future on the calling thread. Window/surface creation stays on the event loop, driver calls stay on the loader thread, and the existing cancellation checks still bracket that work. No new runtime, task, worker thread or nested async wait is needed. Pollster currently just drives this future. Validate graphical startup and close-during-load; headless skips this call. |
| Remove direct `bincode` 2, `rustc-hash` and `windows-sys` | No use in project client source or build script. Protocols use Alkahest. Fyrox still uses bincode 1.3. Windows libraries may still occur transitively. |
| Remove normal `winresource`; target-gate `libloading` to Windows | Only build.rs uses winresource for icon embedding. Keep its build dependency: build.rs references it even on a Linux host cross-compiling to Windows. Only the Windows updater uses libloading directly. Check Windows packaging for the icon and adjacent WinSparkle DLL. |
| Remove game server's direct `tokio-tungstenite` and `serde_json` | No direct users in the game server sources. The client and matchmaking use WebSockets, matchmaking parses JSON, and signaling uses both protocols internally. Declare the game's actual Tokio TCP and async I/O features explicitly. |
| Remove the commented wasm target in `.cargo/config.toml` | No build effect. The active forced Linux target determines output paths used by build.rs, deployment and README. Changing that target needs coordinated path handling. |

[open] Browser WebGPU was an explicit [backend decision](../opengl-context-contention/). Given the author's native-only goal, propose removing wgpu's `webgpu` feature and `BROWSER_WEBGPU` from the allowed mask together, updating README and the repository guidance. The native CLI can keep explaining why `-b webgpu` is unavailable. Preserve Vulkan, DX12, Metal and WGSL. Upstream winit, Fyrox and Matchbox still declare wasm-target dependencies. Cargo.lock covers all targets. Confirm removal from the native graph per target and let Cargo retain required cross-target lock entries. Matchbox/WebRTC is used natively for gameplay.

### Randomness and compatibility

- Linux client tree: rand 0.8.8 comes from Fyrox, 0.9.5 from tungstenite 0.29 via Axum, and 0.10.4 from the application, native RTC and tungstenite 0.30. The [protocol extraction](../shared-network-protocol/) cuts the Axum route. getrandom 0.2.17 serves ring and rand_core 0.6, 0.3.4 serves rand_core 0.9, and 0.4.3 serves current rand, UUID, ahash and the client directly. The obsolete direct JS feature was already removed in the dependency update above.
- Actor IDs use 16 OS-random bytes, reserve zero and check collisions. The machinegun interprets raw random bits as floats before transforming them. Physics uses raw float bits and retries invalid vector lengths. Replacing these calls with uniform floating-point samples would change gameplay. [proposed] Keep the direct getrandom use on the first pass. Switching to an equivalent OS-byte API would remove a manifest edge while leaving getrandom in the tree.
- The shotgun sends a seed in `RemoteMessage::SpawnShotgunShot`. Both local and remote actors regenerate pellet directions using Fyrox's rand 0.8 `StdRng` and range sampler. Keep that path for wire-compatible behavior. A generator migration needs a fixed-seed direction comparison against the baseline, including the sampling algorithm.
- The game server uses Fyrox only for team shuffling and already uses rand 0.10 for team choice. [proposed] Use `rand::seq::SliceRandom` and `rand::rng()` for that shuffle, where only the server chooses teams. Matchmaking uses Fyrox only for `SinkExt` and `StreamExt`; import them from `futures-util` with its sink feature. These changes together with protocol extraction can remove Fyrox from the separate server builds. Client resources/audio still require Fyrox.

### Feature reductions to verify

- Only the client directly enables `image` defaults. The HUD, splash and skybox decoding paths currently receive PNGs. [proposed] Enable only PNG, then exercise all three paths; default formats currently enable AVIF/ravif among other codecs. If a caller starts accepting other formats, its contract must say which formats are supported.
- The client adds winit's `rwh_05` alongside default `rwh_06`. wgpu 30 uses raw-window-handle 0.6.2. [proposed] Drop the 0.5 feature and check window/surface creation on supported platforms.
- The production client should declare Tokio `rt-multi-thread` itself. Isolated package checks avoid dev features and other workspace members masking missing declarations. Likewise inspect server Tokio features: `game_server` invokes `std::process::exit`, while matchmaking actually spawns child processes with Tokio. Preserve each required feature with its caller. The build dependency pair `syn` and `proc-macro2` serves the embedded-asset scanner and remains in use.

[proposed] After approval, compare isolated locked trees before and after and compare stripped or unstripped release binary sizes under identical settings. Verify native launches, package builds and local networking according to the changes. No general dependency upgrade is required.

## 2026-11-24 — Implementation decisions

[accepted] The author approved the native-only browser cleanup, dependency and feature reductions, native clock and loader substitutions, server-only shuffle migration, and retention of the existing gameplay random paths. Keep the active explicit Linux build target for the documented release/deployment layout. The root package version and deployment configuration stay as they are.

### Verification on the uncommitted Linux tree based on baa9800

Compared locked normal dependency trees using the same package, target and counting method as the first pass. The three client entry points share one package graph.

| Application | Linux x86_64 before → after | Windows GNU x86_64 before → after |
|---|---:|---:|
| Client | 513 → 415 | 488 → 393 |
| Game server | 295 → 253 | 295 → 253 |
| Matchmaking server | 114 → 49 | 113 → 48 |

The [protocol extraction](../shared-network-protocol/) removed both server packages, signaling and Axum from the isolated client graph. Neither native client tree contains wasm-bindgen, web-sys, web-time or pollster. Matchmaking no longer builds Fyrox. Both servers still compile separately with their production Tokio features. The new lock adds one local package and removes 58 package/version entries; all retained third-party versions are unchanged. Third-party wasm-target declarations still appear in the cross-target lock.

Measured actual file lengths after `cargo build --release --workspace --locked --offline` under identical settings before and after the changes. The graph counts above are isolated per package; these binaries are from the workspace release configuration.

| Linux binary | Before (bytes) | After (bytes) |
|---|---:|---:|
| game-client | 64,531,144 | 62,551,176 |
| game-client-with-windows-command-line | 64,620,848 | 62,642,224 |
| game-client-without-autoupdate | 64,596,640 | 62,617,944 |
| game_server | 12,453,864 | 12,429,424 |
| matchmaking_server | 2,200,072 | 2,206,360 |

- Linux release workspace build, workspace bins check, production bins checks for each app separately, and Windows GNU workspace bins check completed without compiler warnings. The Windows resource object contains icon and group-icon resources, and the copied DLL is byte-identical to the bundled WinSparkle DLL. No Windows executable was run. The macOS arm64 client tree still needs an uncached Apple crate (`block2`); macOS execution was not verified.
- The workspace test run passed 32 tests in each of the three client bins. Each also had nine local transport tests fail on socket permissions. All 24 vendored signaling tests failed at bind with the same `PermissionDenied`. All three dedicated wire tests passed with the default test-thread stack after the [debug codec fix](../shared-network-protocol/). No production endpoint was contacted.
- With isolated user settings and a loopback-only connection configuration, the no-updater headless client reached `engine: ready` and ran until the 12-second timeout. The sandbox also restricted the gilrs hot-plug monitor. Native `-b webgpu` reported supported options and exited 2.
- A separate probe with the release PNG-only image library decoded all 58 embedded images referenced by the Rust sources, including the splash, HUD and six skybox faces. A dedicated OS thread ran Tokio's `block_on` through Vulkan adapter/device creation on llvmpipe. The real windowed run stopped at winit's `XOpenDisplayFailed`. Actual presentation and closing the window during loading remain unverified here.

[accepted] 2026-11-24 — The author reviewed the result and confirmed closure.
