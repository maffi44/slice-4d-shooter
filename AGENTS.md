# AGENTS.md

Guidance for coding agents working on **Slice: 4D Shooter**, a native multiplayer FPS in true 4D space (Rust, wgpu/WGSL SDF raymarching, SDF physics, actor logic, WebRTC/Matchbox). Communicate with the author in Russian. `CLAUDE.md` is a local import of this file.

## Working principles

- Prefer the simplest direct design with one clear responsibility per system or actor. Introduce a trait, wrapper or layer when it provides a concrete benefit (ownership, invariants, cleanup, reuse), not merely indirection. A useful ownership wrapper is not a forbidden abstraction.
- Minimize input-to-photon and input-to-network latency; avoid frame queues and unnecessary hot-path work. Prefer ownership, channels and atomics over unnecessary locks, but thread-safe operations alone do not guarantee coherent multi-step updates.
- Avoid unnecessary allocations/copies in tick/frame paths. Reuse buffers (`clear`, appropriately reserved capacity), borrow or iterate when practical. `mem::take` loses reusable capacity if the empty replacement is refilled; moving a buffer through a lifecycle that returns/reuses it can be appropriate. Choose `pop`, `drain` or a queue according to required ordering, not an assumed universal speed rule. Assess actual costs before adding a complicated optimization; this is not a claim that all current hot paths allocate nothing.
- Keep established names (`Messenger`, `Actors`, `simulate`, `GlobalSettings`, intentional `recieve_message`) unless a concrete reason warrants change. Preserve deliberate capabilities: headless, temporarily unused 2D/3D content, nested-world API and `SpawnControlledActor`.
- Organize files by responsibility. Split mixed responsibilities into logical submodules when helpful; line count alone is not a reason to split. Avoid fragmentation and forwarding-only wrappers. See [logical module splitting](issues/programming/logical-module-splitting/).
- Discuss substantial architectural changes before implementation, explain trade-offs and stage work so the game stays playable. Once approved, proceed without step-by-step confirmation. Keep unrelated large work in separate issues.

These principles come from the [refactoring context](issues/programming/engine-refactoring-world-isolation/) (Part I, §19, §23 and §24.9) and the 2026-11-16 review response. The latter reopened message-ordering and world-preparation assumptions; see their tasks before treating old decisions as binding.

## Scope, ownership and verification

- The author works on the same branch. Preserve unrelated changes/deletions; ask if their ownership affects your task. Never commit unless asked.
- Finish authorized work fully: implementation, logical module placement, imports, obsolete code cleanup, all affected bins, documentation and task history. Check README every task; update it when behavior, build commands, configuration or documentation organization changes.
- Fix clear local bugs encountered in scope, explaining and verifying them. Ask before uncertain gameplay/protocol changes or substantial scope expansion. Distinguish leftovers from intended capabilities; preserve uncertain features and report them.
- Find root causes and explain technical subtleties candidly. Answer architectural questions with your own assessment, distinguishing evidence, inference, proposal and accepted decision.
- Keep builds free of compiler warnings. Remove real leftovers; use `#[allow(dead_code)] // reason` for intentionally retained APIs/content. Do not suppress warnings indiscriminately.
- Verify according to the change. For engine/gameplay changes: release workspace build, workspace tests, headless and windowed runs, and Windows GNU bins check for Windows-specific code. For rendering changes compare images when feasible. Record what ran, platform/revision or uncommitted state, outcomes and limitations. Documentation-only edits need link/content checks, not a game rebuild.
- Never connect automated runs to production servers. Use local services or report verification limits.
- Only targeted dependency updates (`cargo update -p <crate>`) when requested; never a blanket update. Do not change the root product version or licenses without explicit instruction.

## Repository map

| Location | Responsibility |
|---|---|
| `src/engine.rs`, `src/engine/` | Engine, main loop, Transform |
| `src/engine/shared_systems/` | Shared input, audio, UI, settings, timing, transport, GPU/render scheduler |
| `src/engine/world.rs`, `src/engine/world/` | Per-world actors, messenger, physics, time, descriptor, effects |
| `src/actor/` | Game actors and devices; player gameplay constants in `main_player/constants.rs` |
| `game_server/` | Game server (currently relays client-authoritative gameplay); exports `client_server_protocol` |
| `matchmaking_server/` | Matchmaking server; exports `matchmaking_server_protocol` |
| `matchbox_signaling_modified/` | Modified third-party crate; avoid unrelated edits |
| `src/assets/`, `media/`, `dll/` | Maps/embedded assets, branding, WinSparkle |

Document module-placement exceptions both at the module declaration and here. Current exception: `shared_systems/render/world_render_system.rs` holds the per-world renderer because it shares render-thread internals (`WorldGpu`, slots, `WorldRenderShared`) with the scheduler; moving it to `world/` would expose internals.

Read [Engine architecture](docs/engine-architecture.md) for execution order, ownership, actor extension steps, map lookup, rendering and networking. `Engine::new(headless).run(level, backend)` is the entry point. Worlds own simulations, actors may own nested worlds, shared systems live once per app. Actors message other actors through `Messenger` and access shared systems directly. Current LIFO delivery is under review; do not assume state updates commute: [message ordering](issues/programming/message-ordering/).

## Build and run

```bash
cargo check --workspace --bins --locked
cargo build --release --workspace --locked
cargo test --workspace --locked
cargo check --target x86_64-pc-windows-gnu --bins --locked
cargo run --release --bin game-client -- -l capture_the_flag_map
cargo run --release --bin game-client -- -b vulkan
cargo run --release --bin game-client-without-autoupdate -- --headless
```

- `.cargo/config.toml` forces Linux target `x86_64-unknown-linux-gnu`; output is `target/<target>/<profile>/`. Other platforms need an explicit target. Windows GNU checking needs its toolchain; MSVC cannot be checked here without its C toolchain (`ring`).
- Linux dependencies: `build-essential pkg-config libasound2-dev`. See [README](README.md) for platform/server commands and hosting.
- Three client bins: shipped `game-client` (Windows GUI subsystem, updater), `game-client-with-windows-command-line` (Windows console, updater), `game-client-without-autoupdate` (console, no updater). Keep all compiling. CI currently builds only the shipped client.
- CLI: `-b/--backend gl|dx12|vulkan|metal`, `-l/--level <name>`, `--headless`, `-v/--version`; default map `capture_the_flag_map`. Headless has no window, GPU, rendering or sound device. First graphical launch compiles shaders and can be slow.
- Windows branch of `build.rs` embeds the icon and copies `WinSparkle.dll` next to the exe. Wasm support was removed in `f3e2128`; remaining browser dependencies/configuration are tracked in [cleanup](issues/programming/dependency-and-build-cleanup/).
- Hotkeys: `i`/`O` render scale, `P` shadows, `G` online (HUD still says P), `J`/`K` sensitivity, `N`/`M` volume. User preferences load next to the exe first, then platform config (`~/.config/slice4d/user_settings.json` on Linux); connection settings are embedded, overridable next to the exe.
- Existing automated tests are in vendored Matchbox, not the client/servers; they need local sockets. Their success does not verify gameplay. No standalone test-suite or performance-instrumentation task was requested in the 2026-11-16 review response.

Known limitations: [allocator](issues/programming/global-allocator/) documents blink-alloc direct-mode realloc problems (client excludes debug, server excludes tests); [map compatibility](issues/programming/maps-out-of-sync-with-parser/) documents maps that fail loading; [tick scheduling](issues/programming/engine-tick-rate/) documents accumulated wake-up lateness. Consult these before diagnosing related failures.

## Releases

`.github/workflows/deploy.yml` runs on every push to `main` and supports manual dispatch: Windows installer/release, WinSparkle appcast/Pages, and EC2 server rebuild/restart. Day-to-day work is on `dev`. A root `Cargo.toml` version bump can push an update to Windows players. Client and server protocol/deployed version must agree; server package versions are not the product version. No release/deploy or version bump is implied by ordinary maintenance. See [CI separation](issues/programming/ci-validation-and-release/).

## Issues and decision context

Start with [issues/README.md](issues/README.md), then read the relevant main task. Issues cover all project areas, under `issues/<area>/<stable-slug>/`. Programming is one area; add others when needed.

Each task has a concise main file containing the problem, current decisions, scope, status, priority, open choices and completion criteria. Put extended rationale, discussion, rejected variants, experiments and implementation history in a sibling `-context.md` file when needed. Do not create empty context files merely for symmetry. Read the relevant context when investigating why a choice was made, changing it or needing details; do not load every historical discussion for every task.

### Naming and stable links

- The directory slug is a permanent address: e.g. `issues/programming/global-allocator/`. Do not rename it for priority or completion.
- Open task: `global-allocator.md`; context: `global-allocator-context.md`. No date in open filenames; creation dates stay in metadata.
- High priority: `high-priority-<slug>.md` and `high-priority-<slug>-context.md`. Also set **Priority: High** in the task and index. Normal and long-term priority need no filename prefix; record them in metadata/index. Preserve context pairing when priority changes.
- Completed task: `done-YYYY-MM-DD-<stem>.md` and `done-YYYY-MM-DD-<stem>-context.md`, where `<stem>` includes `high-priority-` if present. The date is **completion**, not creation. Rename only after the author confirms completion, even if implementation is ready.
- Cross-task/document/source-comment references point to the stable directory, with the role/section described: “decision rationale in this task's context, §19”. The directory shows the current pair, including completion/priority in filenames. This avoids repository-wide link changes at completion and works without symlinks or redirect stubs.
- Inside the pair, link directly to the sibling filename and update those local links when renaming. If a precise cross-file anchor really requires a direct filename, update that exceptional link at rename; stable directory links are the default. Keep the index status current. Neither completion nor priority changes require moving directories.

### Context chains and decision history

State a decision and its short reason where it is used; link to the task/context containing the wider argument. That context can link to older contexts instead of copying them. Example: JSON actor descriptions avoided mandatory actor cloning and premature world side effects ([refactoring context](issues/programming/engine-refactoring-world-isolation/), Part I and §12); activation-time resource loading now motivates [async world preparation](issues/programming/async-world-preparation/), which links to [resource-system context](issues/programming/resource-system/).

Follow only relevant links. Before reversing an earlier decision, explain which premise changed or which new evidence invalidates it. Keep intermediate ideas and rejected variants; mark them superseded with a reason, rather than erasing them. Use ✅ accepted, 💡 proposed, ❓ open, ❌ rejected/superseded. The main task is the current summary; context is historical evidence, not a second conflicting specification. Keep a chronological discussion/implementation log there as work proceeds. Date events and preserve author preferences as preferences until a design is actually approved.

## Conventions and licensing

- Follow the edited file's style (often Allman braces and explicit matches). Use correct English in code identifiers, comments and diagnostics; discuss with the author in Russian.
- Keep intentional `recieve_message`. JSON keys (`frenel_color`, `sticiness_negative_*`) and protocol names (`FlagStatus::Droped`, `DropedFlag`, `YouHitedMe`) require compatibility work to rename; do not “correct” them casually.
- Map diagnostics: `Wrong JSON map format: <what's wrong>`.
- New source files (`.rs`, `.wgsl`, Cargo manifests, CI/installer scripts) start with only the two-line copyright header, using the appropriate comment syntax: `Slice 4D Shooter - the first multiplayer shooter set in 4D space` / `Copyright (C) 2023-2026  Timofei Molokov`. Do not add per-file license notices.
- Code: AGPL-3.0-or-later (`LICENSE.txt`, manifest SPDX identifier). Original assets: CC BY-SA 4.0. `media/`: All Rights Reserved; name/logo are trademarks. Third-party licenses remain intact (WinSparkle, modified Matchbox and sounds). Update `CREDITS.md` when adding sounds; README describes licensing. Do not change licenses unless asked.
