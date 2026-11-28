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

## 2026-11-25 — Joint first stage for author review

[proposed] The author suggests one playable stage for [render resources](../render-resources/) and pass scheduling. Its end-to-end example is a player displaying a live scanner in the current HUD. The next stage replaces the UI frontend with Fyrox and exercises multiple destinations. The earlier [Fyrox texture probe](../fyrox-ui-gpu-textures/) covered GPU image consumption. The two existing task addresses keep their history and track the same first-stage demonstration. All contracts below are proposals on dev at `d97ac5e`.

### Actor and world

Extend the existing optional `WorldRenderSystem` with the world-bound registration endpoint. Lend it through `WorldContext` in activation, message delivery, spawn and tick callbacks. `World` owns its slot registration and optional mutable HUD. Actors create effects in `on_added_to_world`. A prepared spare actor has no live targets or registrations. An actor owns its producer handle. Two HUD images may hold the same copyable local image key. An actor can drop its handle while its world continues running.

```rust
// Proposed actor API, shown for one projection.
let zx = render.create_scanner(ScannerPlane::ZX, size);
ui.set_image(LeftScannerDisplayBlue, zx.image())?;
ui.set_image(LeftScannerDisplayRed, zx.image())?;
self.zx = Some(zx);
```

Use an opaque local key containing the owner and a non-reused instance ID. Validate the owner when connecting a widget. Only `World::output_texture_id()` is an inter-world image source. A render-thread binding resolves a key to a view and target generation and refreshes on replacement or retirement. A missing local producer displays transparent pixels. A raw retained wgpu 30.0.2 `TextureView` keeps the texture alive after ordinary handle drops, so an older view could still show frozen pixels and retain GPU memory. With a key the owner controls invalidation even when a widget lives longer. Retire cached bind groups too. Work already submitted may finish using its original GPU resources.

### Submission and ordering

`register_world` reserves a stable ID. Its endpoint sends addressed edits through the existing shared FIFO. A new world's initial scene, ready actor effects and optional HUD become drawable together when activation finishes. This avoids exposing a partially assembled world if the render thread runs during actor callbacks. A parent reserves its position before activating children. Use reverse reservation order for encoding. Validate every inter-world image binding when it is made: the producing slot must run before the consumer. This supplies the child-to-parent case and excludes cycles. An arbitrary order between unrelated worlds can be revisited when there is a caller for it.

The render loop owns a dense vector of slots. A slot owns its cadence counter, targets, prepared scene, ordered local draws and optional HUD draws. The ID stays stable when slots move. Keep ordered add/remove or replacement edits small and apply related changes together between frames. A group added after activation can include a producer and its widget binding. A pipeline worker reports readiness against the original live ID and configuration revision so late completions cannot revive a dead or superseded effect. On dropping the registration, retire the slot as a unit. Process lifecycle edits even for paused slots.

A prepared draw holds its pipeline, bindings, geometry and destination. The loop records the draw into the current frame encoder. In the locked wgpu API `Queue::submit` consumes each `CommandBuffer`. A reusable `RenderBundle` can record draw commands within a compatible render pass if profiling later warrants it. Every due world contributes to one frame submission under the existing idle-GPU gate. Keep the existing hidden/lost-surface upload flush for the single-window stage.

Changing a plane, uniform or widget rect updates data. It does not rebuild the pass list. For new multi-field inputs, publish the latest complete value using bounded storage (e.g. one short swap under a per-instance mutex when the value changes). Read it at the due draw and upload if changed. The scanner's plane is fixed per instance today and its moving data already comes from the world's dynamic uniform. Current multi-buffer scene uploads retain their [separate coherence limitation](../gpu-data-grouping/). A live producer draws every due world frame. An on-demand producer draws on first use, explicit invalidation or resize if a caller needs that policy.

### Targets, effects and legacy HUD

Proposed default: scene at `resolution × scale`, completed output at `resolution` for every rendered world, including the root. World-local UI coordinates use that output size. Upscale and apply bloom to the scene once, then overlay the HUD. The presenter samples the resulting texture with a plain blit into the surface format. This makes a paused root, a nested world and a root sampled by another consumer obey the same output contract. The extra RGBA8 output at 1920×1080 costs 7.91 MiB per rendered world plus the full-screen transfer for presentation. Direct root composition saves traffic at the cost of a second output contract or a conditional path if a caller later samples or pauses it.

Produce scanner textures before HUD composition. The obstacle-course minimap has an additional ordering constraint. Its image currently blends into the low-resolution scene before bloom. To preserve that appearance draw its local image, blend it over the scene, then run upscale/bloom. Its producer reads world buffers and geometry. Any effect that samples a rendered image writes to a separate attachment. The [resource task](../render-resources/) records the shader inputs and map specialization.

Use built-in typed effect factories with embedded WGSL and explicit Rust uniform layouts, input bindings and target formats. A factory returns the same prepared draw representation regardless of which effect supplied the bindings. A new effect extends its own factory, shader and gameplay caller. The frame loop and UI image path stay common. The per-device preparation cache shares compatible pipelines, with a prepared geometry identity for generated variants. Run generation and driver compilation on a loading worker, report validation failure with effect and variant, and keep a placeholder while pending. The existing resource loader can supply a future file-backed shader or image. Live targets are per activation even when two instances use the same compiled shader.

Split the current `UISystem` at the ownership boundary. Keep image assets, pipelines and device geometry shared, with mutable widgets, transforms, bindings and draw order per participating world. The legacy adapter takes a local image as another source for its image draw. Preserve red/blue parent placement, layer order, UVs and alpha. Move UI publication out of `EngineSharedSystems::post_simulate` to the owner. Headless and UI-less callbacks skip visual work while still executing gameplay and message operations currently mixed into the same functions (e.g. `process_w_scanner`). This stage needs no new general widget tree.

### Frame and acceptance example

With a child at rate 3 and a parent at rate 1, eligible frames 1 and 2 use a placeholder until the child first draws. On frame 3 the parent sees the child's scene, scanner and HUD in the same submission. On 4 and 5 it sees that completed image. At rate 0 keep the last image and keep advancing simulation when the owner calls it. On resize keep the previous complete generation until the next due draw has encoded producer and composition, then expose the replacement for later consumers in that submission. Before a root has output, show the splash. On teardown invalidate local images and exported IDs, ignoring delayed edits and returning placeholders to surviving consumers.

Review with two projections, two consumers of one projection, actor removal, child output presented in a parent HUD image, pause/resize/resume, root replacement, splash and headless. Compare the current HUD at full and reduced scale, including live scanner edges and the obstacle minimap before bloom. The [map compatibility task](../maps-out-of-sync-with-parser/) describes the local fixture constraint. Fyrox input, clipping, window routing and surface ownership remain part of the later [UI migration](../fyrox-ui/).

## 2026-11-25 — Author feedback on publication and optional upscale

[superseded] Requiring a full-resolution composition target in every world would charge a small offscreen producer for an upscale it does not need. [accepted] The world chooses the passes that produce its exported texture. With upscale the scene draws at scaled size, the world upscales to nominal size, then draws its UI. With direct output the world draws the UI onto the scaled scene target. A world with no UI can omit those draws. A normal presentation draw only displays the finished texture.

[proposed] Put a default `with_upscale` on the descriptor, defaulting existing maps to their current upscale/bloom behavior. Let the caller override it in `WorldRenderConfig` when creating a smaller view of the same map. At nominal 1920×1080 and scale 0.5 the two modes publish 1920×1080 and 960×540 respectively. Presentation is literally 1:1 only when the output and physical destination sizes match. A plain full-window sampling draw can stretch the direct output if the caller deliberately selects it for a larger window. For a direct raymarch output displayed as a world image, ensure opaque sampling or write opaque alpha in the direct variant: the scene shader writes luminosity into alpha for bloom. Keep local HUD textures' normal alpha. The existing upscale applies bloom even at scale 1. Compare both modes and ensure no bloom is applied twice.

[proposed] Reserve the world ID and an empty slot during activation. Empty slots incur no draw or target allocation. Publish once after initial actor callbacks and subsequently when structure changes. The simulation side is the sole editor of pass order. A whole immutable list, e.g. `Arc<[Pass]>`, can be moved through the shared FIFO and retained by the render thread. Publish once for related edits at the end of a simulation step. Cloning a wgpu handle shares an allocation but costs reference-count operations; sharing a published list or its immutable records avoids per-frame cloning. For a burst of structural changes, coalesce pending snapshots for a live world if necessary, preserving registration, teardown and presentation ordering. Include logical render-target IDs where a resize can change views. Binding resolution and generation checks remain necessary even if the pass list is unchanged.

[proposed] Treat each raster pass as a destination with clear/load semantics and an ordered sequence of draws. A single fullscreen draw and a list of HUD draws fit the same shape. `render_rate` and its frame counter already express the proposed “cadence”. A target is the texture used as a render attachment via its current view. Keep Fyrox's future clip, geometry and draw-data requirements open in the [UI task](../fyrox-ui/). For verification, use a direct-output child at rate 3 under an upscaled parent at rate 1, then pause, resize, resume and retire it. In addition to image order check the actual output sizes, absence of an extra direct-mode target, clear/load order and opacity when displaying the child as an image.

## 2026-11-25 — World output and HUD order

[accepted] The XWZ minimap is an ordinary HUD image after bloom, as selected by the author. This supersedes the proposed pre-bloom comparison. Direct mode composites onto the scaled scene. The root performs one plain presentation draw; child outputs use the same finished image. The former obstacle-course levels are archived; retain the actor and effect for BSP-compatible content.

[accepted] Every rendered world reserves an empty slot before callbacks. One publication binds the complete scene, ready producers and UI; ordinary buffer writes keep the plan. Local producer keys validate the owning world, and world-output keys may only point toward a later registration. A missing image skips its widget. Resizing a paused world preserves its last complete texture until its next due draw. UI bindings track the producer's target generation. The render loop keeps the existing idle-GPU gate and single submission.

### Verification

Uncommitted Linux x86_64 build based on `d97ac5e`, 2026-11-25. The ignored GPU test was run explicitly on Mesa llvmpipe. CTF actor activation published both scanner planes. For deterministic readback, solid blue and green scene inputs then identified the parent and child. A pixel in the parent's widget was blue before the child's first due frame, green once the child drew and blue after removal. The nominal 32×24 parent retained its full output. A half-scale direct child kept its 16-pixel width while paused and switched to 24 on its first resumed frame. The parent re-bound the new generation.

Release workspace build and Windows GNU bins check passed. The headless client loaded CTF. The workspace suite's local-socket tests failed at bind with `EPERM` in the sandbox (24 vendored signaling cases and nine client transport cases per client bin). The remaining client and protocol tests passed. Window startup could not connect to the X display, and a private compositor could not bind its session bus in this environment. Visual acceptance of the live window remains with the author.

### CTF framebuffer comparison

Compared the working tree with `d97ac5e` using separate offscreen capture harnesses and the same llvmpipe adapter, scene, viewport and output format. With 160×120 output and 0.5 scene scale, 19,197 of 19,200 RGBA pixels matched exactly. The three differences were on the same row near the top. At scale 1 with the red scanner displays visible and the same radius and intensity uploaded to both renderers, 19,126 pixels matched exactly. Eight pixels differed by more than two channel levels. These captures exercise the scanner and composition path at a small viewport; they do not measure frame latency or replace a windowed playthrough. The capture runs used Mesa's `GALLIVM_PERF=nopt` to reduce software shader compilation time.
