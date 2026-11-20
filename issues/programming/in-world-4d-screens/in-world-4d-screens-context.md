# Context: In-world 4D screens

[Current task](in-world-4d-screens.md). This file preserves discussion and historical proposals; the current task states the latest scope and decisions. Dates below describe discussions, not completion.

## Preserved history (before the 2026-11-16 documentation migration)

The following original issue is preserved in full. Its old filename rules, status summaries and “final” architecture headings describe their original rounds and are superseded by the current task and AGENTS.md where they disagree. Section numbers remain available for older references.

# In-world 4D screens: showing a texture on geometry inside 4D space

- **Created:** 2026-11-12
- **Status:** Backlog, **preliminary ideas only**. Not a spec. **Not part of the current engine refactoring.**
- **Related:** `issues/programming/engine-refactoring-world-isolation/`. That refactoring makes it possible to create nested `World`s, render each into its own GPU texture, and get that texture (§9.7 there, formerly "Q5"). This issue covers **displaying** such a texture (or a static one) on a screen object that lives inside the 4D game space.

Legend: [accepted] decided · [proposed] proposed · [open] open · [rejected] rejected.

## 1. Goal

The in-game tutorial (see the refactoring issue §1) needs a **screen inside the level** showing another `World`: a 2D slice of a 3D world, as an analogy for the player's 3D slice of 4D space.

Requirements from the author:
- [accepted] The screen is a **real object in 4D space**, not a 2D overlay. Its image must respect its position in space and be **occluded** by other objects correctly.
- [accepted] The displayed texture can be either:
  - an ordinary **static texture**, or
  - the **output texture of another `World`**.

  From the screen's point of view there should be no fundamental difference between the two.
- [accepted] Out of scope for the refactoring. The refactoring only has to provide a GPU texture per world that other GPU resources / pipelines can use.

## 2. Preliminary rendering scheme (author's idea, 2026-11-12) [proposed]

1. **Render the SDF scene as now**, but also produce a **depth texture**: the depth of the visible 3D slice after the SDF raymarch.
2. **A separate pass renders the screens** as ordinary rasterized geometry:
   - each screen is a quad of **two triangles**
   - the vertices have **4D coordinates**
   - each screen has its own **4D transform**, applied in the **vertex shader**
   - the quad has ordinary **UV coordinates**, `(0,0)`…`(1,1)` at the corners
   - the **fragment shader** samples the assigned texture (static or a world's output)
3. The pass uses the **depth texture from step 1**, so screens are correctly occluded by the main SDF geometry.

## 3. Thickness along the 4th dimension [proposed] / [open]

An infinitely thin 2D plane in 4D would leave the 3D slice almost as soon as the player moves along W. So the screen must have some **extent along the extra dimension**. Two variants so far:

- **Variant 1 — constant size within a thickness range.**
  - The screen is a 3D volume in 4D space with a thickness along the extra dimension.
  - While the player's slice is inside that range, the screen is visible at **constant size**.
  - Outside the range it **disappears abruptly**.
- **Variant 2 — size depends on the slice position.**
  - The 4D geometry is chosen so that its 3D cross-sections give a screen of **varying size**: as the player moves along W, the screen appears small, grows to its maximum size, then shrinks and disappears.
  - Something like a **4D double pyramid** (a pyramidal shape with two apexes along the extra axis). The author first thought of a tetrahedron, but a double pyramid is the closer description.

## 4. Open problem: arbitrary 4D rotation [open]

How to implement thickness / size variation correctly when the screen has an **arbitrary 4D rotation**, or is viewed at an angle?
- Simply scaling the quad by the W coordinate is probably not enough once the screen is rotated in 4D.
- The correct answer is the **intersection of the 4D object with the current 3D slice**, which must be computed for arbitrary orientation.
- Not thought through yet. **No implementation method is decided.**

## 5. Notes from the refactoring discussion (context)

- The refactoring will expose each world's output via an id (`WorldTextureId`) that the render thread resolves to a `TextureView`, with a placeholder when the texture is missing (headless, destroyed, not rendered yet, being recreated). See the refactoring issue §9.7 / §20.
- The render loop already renders child worlds before parents (reverse registration order), so a child's texture is ready when the parent's screen pass samples it.
- A texture "generation" counter lets the consumer rebuild its bind group after the child texture was recreated (resolution/scale change).
- Earlier options listed for "Q5" in the refactoring issue:
  - an SDF "screen" object inside the raymarch shader (texture binding + UV projection, fixed slots with placeholder) — not chosen; the author now prefers rasterized geometry + depth test
  - a 2D overlay via the UI renderer — [rejected] rejected: the screen must be an in-world object
- The current raymarch shaders don't output depth. Producing depth (e.g. writing `frag_depth` or a separate depth/R32F target) is part of this task.

## 6. Open questions

- Variant 1 vs Variant 2 (or something else) for the W-thickness.
- Correct slicing under arbitrary 4D rotation.
- Depth format and how the raymarch shader writes it (and its cost).
- Lighting/shading of the screen (unlit? affected by the level's fog/colour grading?).
- How a screen actor declares its texture (`VisualElement` field with a `WorldTextureId` or a static texture handle from the future Resource System).
- Maximum number of screens per world / per frame.

## 7. Discussion

1. **2026-11-12. Author (round 4 of the refactoring discussion).** A 2D overlay doesn't fit; the screen must be a full 4D object with correct occlusion. Proposed the depth-texture + rasterized-quad scheme, 4D vertices and transform in the vertex shader, UVs, static or world texture, W-thickness with two variants (constant size range vs double-pyramid size variation), and raised the arbitrary-rotation problem. Asked to split this into a separate issue and keep it as preliminary discussion, not a spec.
