# Frame queue depth and surface presentation — context

## 2026-11-23 — One-frame queue experiment

- [accepted] The current scheduler prioritizes fresh input. The author previously found that queuing frames smoothed motion but delayed visible reactions. The rationale for the current policy is recorded in the [refactoring context](../engine-refactoring-world-isolation/), §2.4.
- [open] The author reports that smooth motion with the current policy requires a high render rate and high GPU load. An earlier trial that kept roughly two frames buffered looked smoother, but older frames could pile up. The exact queue depth at each stage and the resulting delay were not measured in this report.
- [proposed] Test a queue bounded to one additional frame and try a lower FPS cap. The reason to revisit queueing is the possibility of retaining smooth motion with less GPU work. The acceptable input latency cost remains open. [Research task](vsync-research.md)

## 2026-11-23 — Player control over graphics settings

- [open] The author wants players to be able to choose a balance between responsiveness and smoothness if the experiments establish a meaningful trade-off. Keep the menu design in this investigation because the measurements determine which controls are worth exposing. Preset names, queue-depth controls and defaults are provisional.
- [open] Backend selection raises a separate application question within the same menu. Compare rebuilding graphics resources during a match with saving the preference and restarting the client. Record the player-visible interruption and recovery path before choosing an approach.
