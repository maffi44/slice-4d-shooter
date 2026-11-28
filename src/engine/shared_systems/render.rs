// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



// The device and render scheduler are shared by all worlds. Each rendered world
// publishes a retained scene, local texture producers and an optional HUD. The
// scheduler encodes due worlds in reverse registration order and presents one output.

pub mod render_data;
pub mod pipelines;
pub mod uniform;
pub mod camera;
pub mod raymarch_shader_generator;
pub(crate) mod ui_renderer;
mod gpu_context;
pub(crate) mod render_scheduler;
mod presentation;
pub(in crate::engine) use render_scheduler::WindowPresentation;
pub(crate) mod passes;
pub use crate::engine::world::render::ImageSource;

pub use self::{
    gpu_context::GpuContext,
    render_scheduler::{RenderScheduler, WorldTextureId},
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
