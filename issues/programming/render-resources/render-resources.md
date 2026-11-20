# Render resources for worlds and dynamic UI textures

- **Status:** Open — design pending
- **Priority:** Normal
- **Complexity:** Complex
- **Created:** 2026-11-20

## Goal

Define how a caller describes rendering into a texture, supplies the data its shader needs and uses the resulting texture. Start with the scanner HUD overlay. Gameplay updates its inputs in real time, and the HUD displays the rendered texture over the presented world. A world also renders into a texture using pipelines and per-instance render data. Assess which parts of those two paths benefit from a common representation.

## Required change within this task

[accepted] Remove the minimap shader and pipeline from the map contract. A map supplies only the pipelines needed to render the world itself. Today the map parser reads `main_shader`, while world preparation also selects minimap WGSL and stores an optional `minimap_pipeline` in `WorldDescriptorGpu`. Trace its consumers and move any required HUD rendering to the owner of that effect. Map loading should no longer generate, compile or carry a minimap pipeline for the HUD.

## Design and review

- Describe the caller-facing operations for creating a dynamic texture, updating shader inputs and displaying the output. Use the scanner and a world output as concrete examples before choosing types.
- Decide who owns the pipeline, target texture, buffers and bindings, which resources can be shared and which belong to an instance. Account for resize, teardown and switching the presented world while the scanner is visible.
- [proposed] A common rendering path for worlds and shader-driven UI textures is a proposal. Document which responsibilities coincide and any concrete reason to keep a difference. Coordinate buffer update semantics with [GPU data grouping](../gpu-data-grouping/) and reusable asset ownership with [Resource System](../resource-system/).

## Completion criteria

Map preparation works independently of HUD shaders, including in headless mode. The scanner displays current data for the presented world across transitions. Root and nested worlds retain their rendering controls and texture outputs. Record the agreed caller contract and ownership in the [engine architecture reference](../../../docs/engine-architecture.md), and verify the graphical paths with a visual comparison where feasible.
