// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



// Rendering:
// - `GpuContext` (shared): device, queue, shared layouts and resources for creating GPU objects.
// - `RenderScheduler` (shared): the render loop on its own OS thread. Each frame it encodes all due
//   worlds (children before parents) into one encoder, then the presented world → window → UI,
//   one submit, present. No frame queueing (it doesn't submit while the GPU is busy).
// - `WorldRenderSystem` (per world): the world's render data and GPU buffers (main thread); its
//   pipelines, bind groups and output texture live in the render thread's slot (`WorldGpu`).

pub mod render_data;
pub mod camera;
pub mod raymarch_shader_generator;
mod ui_renderer;
mod gpu_context;
mod render_scheduler;
mod presentation;
pub(in crate::engine) use render_scheduler::WindowPresentation;
// Layout exception (see AGENTS.md, Repository map): WorldRenderSystem is the per-world render part
// and logically belongs to `world/`, but it shares the render-thread internals (WorldGpu, slots,
// WorldRenderShared) with RenderScheduler; moving it would require making those types public.
mod world_render_system;

pub use self::{
    gpu_context::GpuContext,
    render_scheduler::{RenderScheduler, WorldTextureId},
    world_render_system::WorldRenderSystem,
};

use crate::{
    engine::{
        world::physics::dynamic_collider::PlayersDollCollider,
        world::static_object::{
            ColoringArea, StaticObject, VisualWave, VolumeArea
        }
    },
    engine::transform::Transform
};

use net_protocol::gameplay::Team;



pub struct VisualElement<'a> {
    pub transform: &'a Transform,
    pub static_objects: Option<&'a Vec<StaticObject>>,
    pub coloring_areas: Option<&'a Vec<ColoringArea>>,
    pub volume_areas: Option<&'a Vec<VolumeArea>>,
    pub waves: Option<&'a Vec<VisualWave>>,
    pub player: Option<(&'a PlayersDollCollider, Team)>,
    pub child_visual_elem: Option<&'a ChildVisualElement>,
}

pub struct ChildVisualElement {
    pub static_objects: Option<Vec<StaticObject>>,
    pub coloring_areas: Option<Vec<ColoringArea>>,
    pub volume_areas: Option<Vec<VolumeArea>>,
    pub waves: Option<Vec<VisualWave>>,
    pub player: Option<(PlayersDollCollider, Team)>,
}

pub struct RenderQualityData
{
    pub shadows_enabled: bool
}



#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    position_rel: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x2
        ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}


// a full-screen triangle
const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, 3.0, 0.0], position_rel: [-1.0, 3.0]},
    Vertex { position: [3.0, -1.0, 0.0], position_rel: [3.0, -1.0]},
    Vertex { position: [-1.0, -1.0, 0.0], position_rel: [-1.0, -1.0]},
];

const INDICES: &[u16] = &[0, 2, 1];

// the xwz minimap quad (bottom right corner)
const VERTICES_MINIMAP: &[Vertex] = &[
    Vertex { position: [0.2, -0.9 , 0.0], position_rel: [-1.0, -1.0]},
    Vertex { position: [0.9, -0.2, 0.0], position_rel: [1.0, 1.0]},
    Vertex { position: [0.2, -0.2, 0.0], position_rel: [-1.0, 1.0]},
    Vertex { position: [0.9, -0.9, 0.0], position_rel: [1.0, -1.0]},
];

const INDICES_MINIMAP: &[u16] = &[0, 2, 1, 0, 3, 1];
