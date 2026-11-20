# Wait for the next render deadline without a fixed sleep ceiling

- **Status:** Open — not implemented
- **Priority:** Normal
- **Complexity:** Moderate
- **Created:** 2026-11-20
- **Stable address:** `issues/programming/render-deadline-wait/`
- **Context:** [Review evidence and reasoning](render-deadline-wait-context.md)

## Task

Align render-thread waits with the configured frame period while keeping GPU submissions bounded and the command inbox responsive.

## Possible solution [proposed]

Use the time remaining to the next deadline to bound a wait that can also wake on render commands. Specify a bounded retry when a frame is due but the GPU is busy. Keep the idle path from spinning and avoid accumulating frames to submit later.

## Completion criteria

Check the wait calculation at 125, 360 and 1000 FPS. Verify resize/shutdown responsiveness and no busy loop when the GPU is occupied. On available hardware record frame timing and distinguish scheduling limits from GPU/display limits.
