# Schedule scene, texture and UI passes per world

- **Status:** Open — design pending
- **Priority:** High
- **Complexity:** Complex
- **Created:** 2026-11-24
- **Context:** [Initial design and implementation order](world-render-passes-context.md)

## Goal

Let a world produce a complete image, including its UI, before another world samples it. Keep each world's render cadence and the render thread's control over submission. Both the [dynamic texture producer](../render-resources/) and the [Fyrox UI backend](../fyrox-ui/) need a place in this order.

## Proposed first stage

- Group the scene pass, its local producers and UI composition under the owning world's scheduled draw. Register an effect with its world and place it before its first consumer. Encode into the scheduler's frame encoder. Keep the existing world registration and cadence where they already express the dependency.
- Keep presentation as a consumer of the finished output. Preserve a full-resolution root HUD when the scene renders at reduced scale. Agree on the resolution and meaning of the published world output, including the extra target cost, before changing its contract.
- Define what a consumer sees before the first draw, when a producer skips a frame or pauses, and during resize or teardown. Decide the supported dependency directions and how unsupported cycles are reported.

## Completion criteria

Trace two worlds at different render rates. On a child's due frame its scene and UI finish before the parent samples it. Between child draws the parent can reuse its last complete image. Check first use, pause/resume, resize, root replacement, splash and headless. Preserve bounded GPU submissions and verify that a second local pass can register without editing the frame loop. Demonstrate the stage with the current playable client and document the agreed scheduler contract.
