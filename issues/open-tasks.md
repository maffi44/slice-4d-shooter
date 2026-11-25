# Open tasks

Sorted by priority, then status group (awaiting review → decision pending → not implemented → backlog), then creation date. Jump to [tasks by complexity](#by-complexity). Rules: [issue README](README.md). Closed tasks: [closed-tasks.md](closed-tasks.md).

## Programming

| Task | Priority | Complexity | Status | Created | Review |
|---|---|---|---|---|---|
| [Render resources for worlds and dynamic UI textures](programming/render-resources/) | High | Complex | Open — design pending | 2026-11-20 | — |
| [Schedule scene, texture and UI passes per world](programming/world-render-passes/) | High | Complex | Open — design pending | 2026-11-24 | — |
| [Integrate Fyrox UI for world HUDs and multiple windows](programming/fyrox-ui/) | High | Very complex | Open — design pending | 2026-11-24 | — |
| [In-world 4D screens](programming/in-world-4d-screens/) | High | Very complex | Backlog — design pending | 2026-11-12 | — |
| [Global allocator: keep, replace or remove?](programming/global-allocator/) | Normal | Moderate | Open — decision or implementation pending | 2026-11-13 | 12 |
| [Engine tick scheduling](programming/engine-tick-rate/) | Normal | Moderate | Open — decision or implementation pending | 2026-11-13 | — |
| [Maps out of sync with the parser](programming/maps-out-of-sync-with-parser/) | Normal | Moderate | Open — decision or implementation pending | 2026-11-16 | — |
| [Do not silently ignore panics in background threads and tasks](programming/unobserved-panics/) | Normal | Complex | Open — decision or implementation pending | 2026-11-20 | — |
| [Research surface presentation, frame pacing and graphics menu settings](programming/vsync-research/) | Normal | Complex | Open — research pending | 2026-11-20 | — |
| [Define delivery of pending requests when replacing the root world](programming/root-replacement-mailbox/) | Normal | Moderate | Open — design pending | 2026-11-20 | — |
| [Discover bundled destination maps from level JSON](programming/discover-linked-maps/) | Normal | Moderate | Open — design pending | 2026-11-24 | — |
| [Preserve short presses and multiple input bindings](programming/input-button-state/) | Normal | Moderate | Open — not implemented | 2026-11-16 | 4 |
| [Frame server TCP messages and bound reads](programming/server-tcp-framing/) | Normal | Complex | Open — not implemented | 2026-11-16 | 7 |
| [Separate real elapsed time and simulation time if needed](programming/world-time-semantics/) | Normal | Moderate | Open — not implemented | 2026-11-16 | 9 |
| [Separate development CI from release deployment](programming/ci-validation-and-release/) | Normal | Complex | Open — not implemented | 2026-11-16 | 13 |
| [Split mixed responsibilities into logical submodules](programming/logical-module-splitting/) | Normal | Complex | Open — not implemented | 2026-11-16 | 15 |
| [Preserve user settings when the FPS field is invalid](programming/preserve-invalid-user-settings/) | Normal | Moderate | Open — not implemented | 2026-11-20 | — |
| [Wait for the next render deadline without a fixed sleep ceiling](programming/render-deadline-wait/) | Normal | Moderate | Open — not implemented | 2026-11-20 | — |
| [Authoritative server and transport](programming/network-authoritative-server-and-transport/) | Normal | Very complex | Backlog — design pending | 2026-11-12 | — |
| [Review the engine for cache-friendly memory layout](programming/cache-friendly-memory-layout/) | Long-term | Complex | Open — research pending | 2026-11-20 | — |
| [Server side of the engine: many cheap session instances](programming/server-engine/) | Long-term | Very complex | Open — design pending | 2026-11-20 | — |
| [GPU upload grouping and coherent data](programming/gpu-data-grouping/) | Long-term | Complex | Backlog — long-term | 2026-11-16 | 1 |
| [Protect GPU upload buffers from overflow](programming/gpu-buffer-overflow/) | Long-term | Moderate | Backlog — long-term | 2026-11-16 | 5 |
| [Interpolate actor state between simulation ticks for rendering](programming/render-frame-interpolation/) | Long-term | Complex | Backlog — long-term | 2026-11-20 | — |

## By complexity

Estimates cover the work and verification described in each task. Within each group, higher-priority tasks appear first. See the [estimation scale](README.md#complexity-estimates).

### Very simple


### Simple


### Moderate

- [Global allocator: keep, replace or remove?](programming/global-allocator/)
- [Engine tick scheduling](programming/engine-tick-rate/)
- [Maps out of sync with the parser](programming/maps-out-of-sync-with-parser/)
- [Define delivery of pending requests when replacing the root world](programming/root-replacement-mailbox/)
- [Discover bundled destination maps from level JSON](programming/discover-linked-maps/)
- [Preserve short presses and multiple input bindings](programming/input-button-state/)
- [Separate real elapsed time and simulation time if needed](programming/world-time-semantics/)
- [Preserve user settings when the FPS field is invalid](programming/preserve-invalid-user-settings/)
- [Wait for the next render deadline without a fixed sleep ceiling](programming/render-deadline-wait/)
- [Protect GPU upload buffers from overflow](programming/gpu-buffer-overflow/)

### Complex

- [Render resources for worlds and dynamic UI textures](programming/render-resources/)
- [Schedule scene, texture and UI passes per world](programming/world-render-passes/)
- [Do not silently ignore panics in background threads and tasks](programming/unobserved-panics/)
- [Research surface presentation, frame pacing and graphics menu settings](programming/vsync-research/)
- [Frame server TCP messages and bound reads](programming/server-tcp-framing/)
- [Separate development CI from release deployment](programming/ci-validation-and-release/)
- [Split mixed responsibilities into logical submodules](programming/logical-module-splitting/)
- [Review the engine for cache-friendly memory layout](programming/cache-friendly-memory-layout/)
- [GPU upload grouping and coherent data](programming/gpu-data-grouping/)
- [Interpolate actor state between simulation ticks for rendering](programming/render-frame-interpolation/)

### Very complex

- [Integrate Fyrox UI for world HUDs and multiple windows](programming/fyrox-ui/)
- [In-world 4D screens](programming/in-world-4d-screens/)
- [Authoritative server and transport](programming/network-authoritative-server-and-transport/)
- [Server side of the engine: many cheap session instances](programming/server-engine/)
