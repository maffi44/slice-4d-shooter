# Fyrox UI integration evidence

[Current task](fyrox-ui.md)

## 2026-11-24 — Standalone UI and multiple destinations

The project pins Fyrox core, resource and sound to 0.36.2. Use the matching UI release as the first integration candidate. In that version, an independent `UserInterface` owns its node pool, focus state and drawing context. `UiContainer` can hold several interfaces. `UserInterface::draw` returns a borrowed drawing context. A render thread therefore needs owned, reusable draw data or another explicit handoff with a proven lifetime. [Fyrox UI 0.36.2 source](https://docs.rs/crate/fyrox-ui/0.36.2/source/src/lib.rs)

`ImageBuilder::with_texture` takes a Fyrox `TextureResource`. Image drawing emits a texture reference in the command list. The wgpu backend must resolve it to the current view. The drawing commands also carry triangle ranges, brushes, opacity, clip bounds and optional clipping geometry. Text commands identify a font, height and atlas page. [Image source](https://docs.rs/crate/fyrox-ui/0.36.2/source/src/image.rs) · [Drawing commands](https://docs.rs/crate/fyrox-ui/0.36.2/source/src/draw.rs)

[proposed] First experiment: register a GPU-rendered pattern behind an identifiable Fyrox texture resource, display it in an ordinary image alongside clipped text, then resize and replace the target. Check orientation, alpha and generation changes without GPU readback or pixel re-upload. Publish owned draw data from the simulation/UI thread and encode it on the existing render thread. Verify reuse across draws, bounded handoff and safe retirement of referenced textures. The integration experiment has not yet run.

Offscreen interfaces and native windows exercise different engine code. The Fyrox Book demonstrates rendering an additional UI into a texture with the *Fyrox engine renderer*. Slice owns its wgpu device, surfaces and event loop. Today `MainLoop::window_event` ignores its window ID, and `RenderLoop::render_frame` acquires one surface before drawing any worlds. An additional interface alone cannot route input or keep a second window rendering if acquisition of the first fails. [Fyrox Book on rendering](https://fyrox-book.github.io/ui/rendering.html)

[proposed] The world owns its optional UI tree. An active actor owns its subtree's lifetime and holds handles into that tree. A shared wgpu backend encodes each interface into the specified target. For native output, associate each winit window ID with its destination, size, scale and focused interface. Define UI capture versus gameplay input before migrating interactive widgets. Share device resources across compatible surfaces. Verify two offscreen targets first, then two actual windows with independent focus and surface lifecycle.

[open] Agree which gameplay action opens the second native window, what should happen to its world on close, and which resolution to use if one world feeds differently sized views. A two-window demonstration can test the platform contract before choosing a player-facing feature. See the [staged plan](../world-render-passes/).

## 2026-11-24 — Fyrox updated to 1.0.1

At the author's request, all dependencies were updated, and Fyrox core, resource and sound moved from 0.36.2 to 1.0.1. The 0.36.2 source references above must be rechecked against `fyrox-ui` 1.0 before the integration experiment. The resource manager now keeps an in-memory UUID registry. See the update notes in [dependency cleanup](../dependency-and-build-cleanup/).


## 2026-11-24 — GPU texture path checked

The separate research task [GPU textures in Fyrox UI](../fyrox-ui-gpu-textures/) analyzed `fyrox-ui` 1.0.1 and ran an isolated wgpu probe outside the repository. In 1.0.1, `UserInterface::update` fills the public `drawing_context` instead of `draw` returning a borrowed context. An `Image` with `TextureResource::new_render_target` displayed a texture produced only on the GPU, clipped next to text, through a key-to-view registry. Content updates and target recreation did not copy pixels through the CPU. The image-to-texture part of the first experiment proposed above is covered. Gradients, clipping geometry, sRGB handling and the render-thread handoff are still open.
