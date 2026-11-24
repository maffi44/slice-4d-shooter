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
