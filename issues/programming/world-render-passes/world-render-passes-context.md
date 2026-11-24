# Per-world render order and the UI migration

[Current task](world-render-passes.md)

## 2026-11-24 — Initial analysis on dev at 87ea043

The author wants actor-created GPU textures, world-local UI at the owning world's cadence, and multiple UI destinations. The existing [render resources task](../render-resources/) already owns pipeline and target lifetime. The earlier [world isolation review](../engine-refactoring-world-isolation/) explicitly deferred the UI rewrite.

In `RenderLoop::render_frame`, the scheduler acquires its one surface, walks live world slots in reverse registration order, upscales the selected output, draws one HUD into the surface, then submits once. Reverse order covers children registered after their parent. A consumer introduced later can also sample an earlier producer, so registration order alone cannot express arbitrary texture dependencies. A world's `render_rate` counts eligible render frames (0 pauses, 1 draws each time, N every Nth frame). Its simulation advances when its owner calls `simulate`. Tying UI input processing to surface acquisition or equating a simulation tick with a render deadline would change the caller's behavior.

[proposed] Keep a compact ordered set of passes per world. Start with world-local dependencies and child output consumed by its parent. Build or validate order when registrations or bindings change. Consider a general graph only when an agreed caller needs additional edges. A producer reading scene output must write into a different target from the texture it samples.

For example, a child that draws every third eligible frame can encode its scene, scanner texture and HUD in that order. Its parent can then compose the completed child image into its own view in the same submission. On the two intervening frames the parent reuses the previous child image. Registration and removal must remain ordered, even if a newer tick supersedes unpublished visual data.

[open] Today the root HUD stays sharp at reduced scene scale because it runs after upscale. For nested consumers a composed image may require a second, output-sized target. For a root used only for presentation, composing directly to the surface could avoid that allocation. Decide which callers need a sampleable scene alone or a complete image, and at what resolution. On resize the current loop recreates a target *before* checking whether a world is due. Retaining the previous complete output or drawing the replacement before publishing it avoids exposing an unrendered generation.

## Proposed sequence for playable stages

1. Agree on the scanner caller example and verify the Fyrox image-to-GPU mapping in a small integration experiment. This can invalidate an API choice before a scheduler rewrite.
2. Introduce world-local pass order in the current client. Confirm the child/parent sequence and current HUD sharpness using the existing renderer.
3. Use [render resources](../render-resources/) to draw a scanner into an actor-owned target and display it in the current HUD. Move the obstacle-course minimap out of map preparation. Each effect keeps its own shading and data requirements.
4. Use the same texture in a [Fyrox UI](../fyrox-ui/) image, migrate the HUD and exercise independent interfaces, first offscreen and then in two native windows. The window integration can proceed once the backend can target an explicit destination.

[proposed] Keep three linked tasks. A single task would couple proving the drawing order to replacing every HUD caller. A separate issue for pipeline registration would split creation from its first real caller. [In-world screens](../in-world-4d-screens/) still own depth and 4D geometry. [GPU data grouping](../gpu-data-grouping/) and [frame pacing research](../vsync-research/) have independent goals.
