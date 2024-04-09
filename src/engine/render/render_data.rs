mod dynamic_render_data;
mod static_render_data;

use crate::engine::{
    time::TimeSystem,
    world::World,
};

use self::{
    dynamic_render_data::DynamicRenderData,
    static_render_data::StaticRenderData,
};

use winit::window::Window;



pub struct RenderData {
    pub dynamic_data: DynamicRenderData,
    pub static_data: StaticRenderData,
}

impl RenderData {
    pub fn new(world: &World, time: &TimeSystem, window: &Window) -> Self {
        let static_data = StaticRenderData::new(world);
        let dynamic_data = DynamicRenderData::new();

        RenderData {
            static_data,
            dynamic_data,
        }
    }

    pub fn update_dynamic_render_data(
        &mut self,
        world: &World,
        time: &TimeSystem,
        window: &Window
    ) {
        self.dynamic_data.update(world, time, window);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Shape {
    pub pos: [f32;4],
    pub size: [f32;4],
    pub color: [f32;3],
    pub roundness: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct PlayerForm {
    pos: [f32;4],
    empty_bytes: [u32; 4],
    color: [f32;3],
    radius: f32,
    // rotation: [f32; 16],
}

impl Default for Shape {
    fn default() -> Self {
        Shape {
            pos: [0.0, 0.0, 0.0, 0.0],
            size: [1.0, 1.0, 1.0, 1.0],
            color: [0.0, 0.0, 0.0],
            roundness: 0.0,
        }
    }
}

pub struct ShapesArrays {
    pub normal: Box<[Shape; 256]>,
    pub negative: Box<[Shape; 256]>,
    pub stickiness: Box<[Shape; 256]>,
    pub neg_stickiness: Box<[Shape; 256]>,
}

impl Default for ShapesArrays {
    fn default() -> Self {
        let normal = Box::new([Shape::default(); 256]);
        let negative = Box::new([Shape::default(); 256]);
        let stickiness = Box::new([Shape::default(); 256]);
        let neg_stickiness = Box::new([Shape::default(); 256]);

        ShapesArrays {
            normal,
            negative,
            stickiness,
            neg_stickiness,
        }   
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShapesArraysMetadata {
    //normals
    cubes_start: u32,
    cubes_amount: u32,

    spheres_start: u32,
    spheres_amount: u32,

    inf_cubes_start: u32,
    inf_cubes_amount: u32,

    sph_cubes_start: u32,
    sph_cubes_amount: u32,

    //stickinesses
    s_cubes_start: u32,
    s_cubes_amount: u32,

    s_spheres_start: u32,
    s_spheres_amount: u32,

    s_inf_cubes_start: u32,
    s_inf_cubes_amount: u32,

    s_sph_cubes_start: u32,
    s_sph_cubes_amount: u32,

    //negatives
    neg_cubes_start: u32,
    neg_cubes_amount: u32,

    neg_spheres_start: u32,
    neg_spheres_amount: u32,

    neg_inf_cubes_start: u32,
    neg_inf_cubes_amount: u32,

    neg_sph_cubes_start: u32,
    neg_sph_cubes_amount: u32,

    //neg_stickinesses
    s_neg_cubes_start: u32,
    s_neg_cubes_amount: u32,

    s_neg_spheres_start: u32,
    s_neg_spheres_amount: u32,

    s_neg_inf_cubes_start: u32,
    s_neg_inf_cubes_amount: u32,

    s_neg_sph_cubes_start: u32,
    s_neg_sph_cubes_amount: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SphericalAreasMetadata {
    pub holegun_colorized_areas_start: u32,
    pub holegun_colorized_areas_amount: u32,
    pub explode_areas_start: u32,
    pub explode_areas_amount: u32,
}

impl Default for SphericalAreasMetadata {
    fn default() -> Self {
        SphericalAreasMetadata {
            holegun_colorized_areas_start: 0,
            holegun_colorized_areas_amount: 0,
            explode_areas_start: 0,
            explode_areas_amount: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SphericalArea {
    pos: [f32;4],
    color: [f32;3],
    radius: f32,
}

impl Default for SphericalArea {
    fn default() -> Self {
        SphericalArea {
            pos: [0.0, 0.0, 0.0, 0.0],
            color: [0.0, 0.0, 0.0],
            radius: 1.0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BeamArea {
    pos1: [f32; 4],
    pos2: [f32; 4],
    color: [f32; 3],
    radius: f32,
}

impl Default for BeamArea {
    fn default() -> Self {
        BeamArea {
            pos1: [0.0, 0.0, 0.0, 0.0],
            pos2: [0.0, 0.0, 0.0, 0.0],
            color: [0.0, 0.0, 0.0],
            radius: 1.0,
        }
    }
}