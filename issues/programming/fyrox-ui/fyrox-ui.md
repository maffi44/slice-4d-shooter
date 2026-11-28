# Integrate Fyrox UI for world HUDs and multiple windows

- **Status:** Open — design pending
- **Priority:** High
- **Complexity:** Very complex
- **Created:** 2026-11-24
- **Context:** [Versioned API evidence and first integration experiment](fyrox-ui-context.md)

[proposed] Schedule the HUD migration after the combined [world pass](../world-render-passes/) and [dynamic texture](../render-resources/) stage has demonstrated the scanner in the current HUD. The [GPU texture probe](../fyrox-ui-gpu-textures/) already established that Fyrox images can consume these outputs.

## Goal

Give actors access to a world's UI, let them create and remove their widgets, and display static images and live [render textures](../render-resources/). Render each interface into an explicit destination using the [per-world pass order](../world-render-passes/). Support multiple independent interfaces and native windows.

## Design and review

- The shared first stage uses the current HUD to verify ordered raster draws, local image bindings and lifetime. Its UI draw representation is provisional. Revisit it against Fyrox's actual clipping, geometry, texture and cross-thread draw-data requirements while retaining the world-selected output and image ownership contract.

- Give the caller a concrete example from player activation to scanner widget removal. Distinguish actor ownership of a widget subtree from ownership of the Fyrox node pool. Keep simulation and input independent of whether a world draws on a given render frame.
- Prove the standalone Fyrox-to-wgpu path with a static image, text, a clipped widget and a GPU-produced image. Then migrate the current HUD, including scanner views, masked bars, score, tutorial and network status. Preserve layering and input behavior through each playable stage.
- Define how each native window selects a UI and output. Route input, focus, cursor coordinates and DPI by window. Keep platform window/surface creation on the event-loop thread. Consider surface format, resize, minimization, close and loss independently for each destination.

## Completion criteria

Compare the migrated HUD with the current graphical client, including reduced scene scale. Demonstrate independent UI in a root and nested world and in two native windows. Exercise input focus, resizing one window and closing the secondary window while the primary continues. Verify actor removal, root replacement and headless gameplay without per-session GPU or unused widget trees. Document the approved actor, rendering and window contracts.

[accepted] The current HUD backend consumes local image keys and later-registered world output IDs. The XWZ minimap is a normal post-bloom image. Replace the current widget tree and image/bar encoding in this stage while retaining the world plan, resource loaders and producer-before-consumer ordering proven in the [world pass task](../world-render-passes/).
