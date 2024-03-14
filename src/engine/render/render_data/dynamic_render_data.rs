use crate::{
    actor::{
        Actor,
        ActorWrapper
    }, engine::{
        physics::physics_system_data::ShapeType, render::render_data::{
            Shape, ShapesArrays, ShapesArraysMetadata, SphericalArea, SphericalAreasMetadata
        }, time::TimeSystem, world::{static_object::{ColoringArea, SphericalVolumeArea, VolumeArea}, World}
    }
};

use glam::{Mat4, Vec4};
use winit::window::Window;

use super::BeamArea;



pub struct DynamicRenderData {
    pub dynamic_shapes_data: ShapesArrays,
    pub spherical_areas_data: Box<[SphericalArea; 256]>,
    pub beam_areas_data: Box<[BeamArea; 128]>,
    pub other_dynamic_data: OtherDynamicData,

    // frame memory buffers
    frame_cubes_buffer: SpecificShapeBuffers,
    frame_spheres_buffer: SpecificShapeBuffers,
    frame_sph_cubes_buffer: SpecificShapeBuffers,
    frame_inf_w_cubes_buffer: SpecificShapeBuffers,

    frame_coloring_areas_buffer: Vec<SphericalArea>,
    frame_spherical_volume_areas_buffer: Vec<SphericalArea>,
    frame_beam_volume_areas_buffer: Vec<BeamArea>,
}

impl DynamicRenderData {
    pub fn new() -> Self {
        let dynamic_render_data = DynamicRenderData {
            dynamic_shapes_data: ShapesArrays::default(),
            spherical_areas_data: Box::new([SphericalArea::default(); 256]),
            beam_areas_data: Box::new([BeamArea::default(); 128]),
            other_dynamic_data: OtherDynamicData::default(),

            frame_cubes_buffer: SpecificShapeBuffers::default(),
            frame_spheres_buffer: SpecificShapeBuffers::default(),
            frame_sph_cubes_buffer: SpecificShapeBuffers::default(),
            frame_inf_w_cubes_buffer: SpecificShapeBuffers::default(),

            frame_coloring_areas_buffer: Vec::new(),
            frame_spherical_volume_areas_buffer: Vec::new(),
            frame_beam_volume_areas_buffer: Vec::new(),
        };

        // dynamic_render_data.update(world, time, window);

        dynamic_render_data
    }



    pub fn update(
        &mut self,
        world: &World,
        time: &TimeSystem,
        window: &Window,
    ) {
        
        self.frame_cubes_buffer.clear_buffers();
        self.frame_spheres_buffer.clear_buffers();
        self.frame_sph_cubes_buffer.clear_buffers();
        self.frame_inf_w_cubes_buffer.clear_buffers();


        for (_, actor) in world.actors.iter() {

            if let Some(visual_element) = actor.get_visual_element() {

                let transform = visual_element.transfrom;

                if let Some(static_objects) = visual_element.static_objects {
                    for static_object in static_objects {
                        
                        let position = static_object.collider.position + transform.get_position();
                        let size = static_object.collider.size * transform.get_scale();
                        let color = static_object.material.color;
                        let roundness = static_object.collider.roundness;
                        
                        let shape = Shape {
                            pos: position.to_array(),
                            size: size.to_array(),
                            color: color.to_array(),
                            roundness,
                        };
    
                        let is_positive = static_object.collider.is_positive;
                        let is_stickiness = static_object.collider.stickiness;
                        
                        match static_object.collider.shape_type {
    
                            ShapeType::Cube => {
                                if is_positive {
                                    if is_stickiness {
                                        self.frame_cubes_buffer.stickiness.push(shape);
                                    } else {
                                        self.frame_cubes_buffer.normal.push(shape);
                                    }
                                } else {
                                    if is_stickiness {
                                        self.frame_cubes_buffer.neg_stickiness.push(shape);
                                    } else {
                                        self.frame_cubes_buffer.negative.push(shape);
                                    }
                                }
                            },
                            ShapeType::Sphere => {
                                if is_positive {
                                    if is_stickiness {
                                        self.frame_spheres_buffer.stickiness.push(shape);
                                    } else {
                                        self.frame_spheres_buffer.normal.push(shape);
                                    }
                                } else {
                                    if is_stickiness {
                                        self.frame_spheres_buffer.neg_stickiness.push(shape);
                                    } else {
                                        self.frame_spheres_buffer.negative.push(shape);
                                    }
                                }
                                
                            },
                            ShapeType::SphCube => {
                                if is_positive {
                                    if is_stickiness {
                                        self.frame_sph_cubes_buffer.stickiness.push(shape);
                                    } else {
                                        self.frame_sph_cubes_buffer.normal.push(shape);
                                    }
                                } else {
                                    if is_stickiness {
                                        self.frame_sph_cubes_buffer.neg_stickiness.push(shape);
                                    } else {
                                        self.frame_sph_cubes_buffer.negative.push(shape);
                                    }
                                }
                                
                            },
                            ShapeType::CubeInfW => {
                                if is_positive {
                                    if is_stickiness {
                                        self.frame_inf_w_cubes_buffer.stickiness.push(shape);
                                    } else {
                                        self.frame_inf_w_cubes_buffer.normal.push(shape);
                                    }
                                } else {
                                    if is_stickiness {
                                        self.frame_inf_w_cubes_buffer.neg_stickiness.push(shape);
                                    } else {
                                        self.frame_inf_w_cubes_buffer.negative.push(shape);
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(volume_areas) = visual_element.volume_areas {
                    
                    for volume_area in volume_areas {

                        match volume_area {
                            VolumeArea::SphericalVolumeArea(spherical_area) => {
                                let area = SphericalArea {
                                    pos: (spherical_area.translation + transform.get_position()).to_array(),
                                    radius: spherical_area.radius,
                                    color: spherical_area.color.to_array(),
                                };
        
                                self.frame_spherical_volume_areas_buffer.push(area)
                            },

                            VolumeArea::BeamVolumeArea(beam_area) => {
                                let area = BeamArea {
                                    pos1: (beam_area.translation_pos_1 + transform.get_position()).to_array(),
                                    pos2: (beam_area.translation_pos_2 + transform.get_position()).to_array(),
                                    radius: beam_area.radius,
                                    color: beam_area.color.to_array(),
                                };
        
                                self.frame_beam_volume_areas_buffer.push(area);
                            }
                        }

                    }
                }

                if let Some(coloring_areas) = visual_element.coloring_areas {
                    
                    for coloring_area in coloring_areas {
                        let area = SphericalArea {
                            pos: (coloring_area.translation + transform.get_position()).to_array(),
                            radius: coloring_area.radius,
                            color: coloring_area.color.to_array(),
                        };

                        self.frame_coloring_areas_buffer.push(area)
                    }
                }
            }
        }

        let mut cubes_start = 0u32;
        let mut cubes_amount = 0u32;

        let mut spheres_start = 0u32;
        let mut spheres_amount = 0u32;

        let mut inf_cubes_start = 0u32;
        let mut inf_cubes_amount = 0u32;

        let mut sph_cubes_start = 0u32;
        let mut sph_cubes_amount = 0u32;


        let mut s_cubes_start = 0u32;
        let mut s_cubes_amount = 0u32;

        let mut s_spheres_start = 0u32;
        let mut s_spheres_amount = 0u32;

        let mut s_inf_cubes_start = 0u32;
        let mut s_inf_cubes_amount = 0u32;

        let mut s_sph_cubes_start = 0u32;
        let mut s_sph_cubes_amount = 0u32;


        let mut neg_cubes_start = 0u32;
        let mut neg_cubes_amount = 0u32;

        let mut neg_spheres_start = 0u32;
        let mut neg_spheres_amount = 0u32;

        let mut neg_inf_cubes_start = 0u32;
        let mut neg_inf_cubes_amount = 0u32;

        let mut neg_sph_cubes_start = 0u32;
        let mut neg_sph_cubes_amount = 0u32;


        let mut s_neg_cubes_start = 0u32;
        let mut s_neg_cubes_amount = 0u32;

        let mut s_neg_spheres_start = 0u32;
        let mut s_neg_spheres_amount = 0u32;

        let mut s_neg_inf_cubes_start = 0u32;
        let mut s_neg_inf_cubes_amount = 0u32;

        let mut s_neg_sph_cubes_start = 0u32;
        let mut s_neg_sph_cubes_amount = 0u32;



        // packing normal shapes
        let mut index = 0;
        cubes_start = 0u32;

        while let Some(shape) = self.frame_cubes_buffer.normal.pop() {
            self.dynamic_shapes_data.normal[index] = shape;
            index += 1;
        }

        cubes_amount = index as u32;


        spheres_start = index as u32;

       while let Some(shape) = self.frame_spheres_buffer.normal.pop() {
            self.dynamic_shapes_data.normal[index] = shape;
            index += 1;
        }

        spheres_amount = index as u32 - spheres_start;


        inf_cubes_start = index as u32;

       while let Some(shape) = self.frame_inf_w_cubes_buffer.normal.pop() {
            self.dynamic_shapes_data.normal[index] = shape;
            index += 1;
        }

        inf_cubes_amount = index as u32 - inf_cubes_start;


        sph_cubes_start = index as u32;

       while let Some(shape) = self.frame_sph_cubes_buffer.normal.pop() {
            self.dynamic_shapes_data.normal[index] = shape;
            index += 1;
        }

        sph_cubes_amount = index as u32 - sph_cubes_start;


        // packing stickiness shapes
        let mut index = 0;
        s_cubes_start = 0u32;

       while let Some(shape) = self.frame_cubes_buffer.stickiness.pop() {
            self.dynamic_shapes_data.stickiness[index] = shape;
            index += 1;
        }

        s_cubes_amount = index as u32;


        s_spheres_start = index as u32;

       while let Some(shape) = self.frame_spheres_buffer.stickiness.pop() {
            self.dynamic_shapes_data.stickiness[index] = shape;
            index += 1;
        }

        s_spheres_amount = index as u32 - s_spheres_start;


        s_inf_cubes_start = index as u32;

       while let Some(shape) = self.frame_inf_w_cubes_buffer.stickiness.pop() {
            self.dynamic_shapes_data.stickiness[index] = shape;
            index += 1;
        }

        s_inf_cubes_amount = index as u32 - s_inf_cubes_start;


        s_sph_cubes_start = index as u32;

       while let Some(shape) = self.frame_sph_cubes_buffer.stickiness.pop() {
            self.dynamic_shapes_data.stickiness[index] = shape;
            index += 1;
        }

        s_sph_cubes_amount = index as u32 - s_sph_cubes_start;



        // packing negative shapes
        let mut index = 0;
        neg_cubes_start = 0u32;

       while let Some(shape) = self.frame_cubes_buffer.negative.pop() {
            self.dynamic_shapes_data.negative[index] = shape;
            index += 1;
        }

        neg_cubes_amount = index as u32;


        neg_spheres_start = index as u32;

       while let Some(shape) = self.frame_spheres_buffer.negative.pop() {
            self.dynamic_shapes_data.negative[index] = shape;
            index += 1;
        }

        neg_spheres_amount = index as u32 - neg_spheres_start;


        neg_inf_cubes_start = index as u32;

       while let Some(shape) = self.frame_inf_w_cubes_buffer.negative.pop() {
            self.dynamic_shapes_data.negative[index] = shape;
            index += 1;
        }

        neg_inf_cubes_amount = index as u32 - neg_inf_cubes_start;


        neg_sph_cubes_start = index as u32;

       while let Some(shape) = self.frame_sph_cubes_buffer.negative.pop() {
            self.dynamic_shapes_data.negative[index] = shape;
            index += 1;
        }

        neg_sph_cubes_amount = index as u32 - neg_sph_cubes_start;



        // packing negative and stickiness shapes
        let mut index = 0;
        s_neg_cubes_start = 0u32;

       while let Some(shape) = self.frame_cubes_buffer.neg_stickiness.pop() {
            self.dynamic_shapes_data.neg_stickiness[index] = shape;
            index += 1;
        }

        s_neg_cubes_amount = index as u32;


        s_neg_spheres_start = index as u32;

       while let Some(shape) = self.frame_spheres_buffer.neg_stickiness.pop() {
            self.dynamic_shapes_data.neg_stickiness[index] = shape;
            index += 1;
        }

        s_neg_spheres_amount = index as u32 - s_neg_spheres_start;


        s_neg_inf_cubes_start = index as u32;

       while let Some(shape) = self.frame_inf_w_cubes_buffer.neg_stickiness.pop() {
            self.dynamic_shapes_data.neg_stickiness[index] = shape;
            index += 1;
        }

        s_neg_inf_cubes_amount = index as u32 - s_neg_inf_cubes_start;


        s_neg_sph_cubes_start = index as u32;

       while let Some(shape) = self.frame_sph_cubes_buffer.neg_stickiness.pop() {
            self.dynamic_shapes_data.neg_stickiness[index] = shape;
            index += 1;
        }

        s_neg_sph_cubes_amount = index as u32 - s_neg_sph_cubes_start;


        let shapes_arrays_metadata = ShapesArraysMetadata {
            cubes_start,
            cubes_amount,
            spheres_start,
            spheres_amount,
            inf_cubes_start,
            inf_cubes_amount,
            sph_cubes_start,
            sph_cubes_amount,
            s_cubes_start,
            s_cubes_amount,
            s_spheres_start,
            s_spheres_amount,
            s_inf_cubes_start,
            s_inf_cubes_amount,
            s_sph_cubes_start,
            s_sph_cubes_amount,
            neg_cubes_start,
            neg_cubes_amount,
            neg_spheres_start,
            neg_spheres_amount,
            neg_inf_cubes_start,
            neg_inf_cubes_amount,
            neg_sph_cubes_start,
            neg_sph_cubes_amount,
            s_neg_cubes_start,
            s_neg_cubes_amount,
            s_neg_spheres_start,
            s_neg_spheres_amount,
            s_neg_inf_cubes_start,
            s_neg_inf_cubes_amount,
            s_neg_sph_cubes_start,
            s_neg_sph_cubes_amount,
        };

        let mut coloring_areas_start = 0u32;
        let mut coloring_areas_amount = 0u32;

        let mut volume_areas_start = 0u32;
        let mut volume_areas_amount = 0u32;

        let mut index = 0;
        coloring_areas_start = 0u32;

        while let Some(area) = self.frame_coloring_areas_buffer.pop() {
            self.spherical_areas_data[index] = area;

            index += 1;
        }

        coloring_areas_amount = index as u32;

        volume_areas_start = index as u32;

        while let Some(area) = self.frame_spherical_volume_areas_buffer.pop() {
            self.spherical_areas_data[index] = area;

            index += 1;
        }
 
        volume_areas_amount = index as u32 - volume_areas_start;

        let spherical_areas_meatadata = SphericalAreasMetadata {
            holegun_colorized_areas_start: coloring_areas_start,
            holegun_colorized_areas_amount: coloring_areas_amount,
            explode_areas_start: volume_areas_start,
            explode_areas_amount: volume_areas_amount,
        };

        let mut index = 0_usize;

        while let Some(area) = self.frame_beam_volume_areas_buffer.pop() {
            self.beam_areas_data[index] = area;

            index += 1;
        }

        let beams_areas_amount = index as u32;

        self.other_dynamic_data.update(
            world,
            time,
            window,
            shapes_arrays_metadata,
            spherical_areas_meatadata,
            beams_areas_amount
        );
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct OtherDynamicData {
    dynamic_shapes_arrays_metadata: ShapesArraysMetadata,
    spherical_areas_metadata: SphericalAreasMetadata,
    camera_data: CameraUniform,
    beam_areas_amount: u32,
    // empty_bytes1: [f32; 3],
    // empty_bytes2: [f32; 4],
    // explore_w_pos: f32,
    // explore_w_coef: f32,
    stickiness: f32,
    screen_aspect: f32,
    time: f32,   
}

impl OtherDynamicData {
    pub fn update(
        &mut self,
        world: &World,
        time: &TimeSystem,
        window: &Window,
        shapes_arrays_metadata: ShapesArraysMetadata,
        spherical_areas_meatadata: SphericalAreasMetadata,
        beams_areas_amount: u32,
    ) {
        
        let cam_pos;
        let cam_rot;

        let explore_w_pos;
        let explore_w_coef;
        
        if let Some(actor) = world.actors.get(&world.main_camera_from) {
            if let ActorWrapper::Player(main_player) = actor {
                cam_pos = main_player.get_position() + Vec4::Y * main_player.get_collider_radius() * 0.98;
                cam_rot = main_player.get_rotation_matrix();

                explore_w_pos = main_player.get_explore_w_position();
                explore_w_coef = main_player.get_explore_w_coefficient();
            } else {
                panic!("main camera is connected to the actor that is not a Player")
            }
        } else {
            panic!("main camera is not connected to the player")
        }

        // self.explore_w_pos = explore_w_pos;
        // self.explore_w_coef = explore_w_coef;

        self.camera_data = CameraUniform {
            cam_pos: cam_pos.to_array(),
            cam_rot: cam_rot.to_cols_array(),
        };

        self.dynamic_shapes_arrays_metadata = shapes_arrays_metadata;
        self.spherical_areas_metadata = spherical_areas_meatadata;

        self.screen_aspect = {
            let size = window.inner_size();
            size.width as f32 / size.height as f32
        };

        self.beam_areas_amount = beams_areas_amount;

        self.time = time.timestamp_of_main_loop_start.elapsed().as_secs_f32();
    }
}

impl Default for OtherDynamicData {
    fn default() -> Self {

        OtherDynamicData {
            dynamic_shapes_arrays_metadata: ShapesArraysMetadata::default(),
            spherical_areas_metadata: SphericalAreasMetadata::default(),
            camera_data: CameraUniform::default(),
            beam_areas_amount: 0,
            // empty_bytes1: [0.0; 3],
            // empty_bytes2: [0.0; 4],
            // explore_w_pos: 0.0,
            // explore_w_coef: 0.0,
            stickiness: 0.5,
            screen_aspect: 1.0,
            time: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub cam_pos: [f32; 4],
    pub cam_rot: [f32; 16],
}

impl Default for CameraUniform {
    fn default() -> Self {
        let cam_pos = [0.0, 0.0, 0.0, 0.0];
        let cam_rot = Mat4::IDENTITY.to_cols_array();

        CameraUniform {
            cam_pos,
            cam_rot,
        }
    }
}


pub struct SpecificShapeBuffers {
    pub normal: Vec<Shape>,
    pub negative: Vec<Shape>,
    pub stickiness: Vec<Shape>,
    pub neg_stickiness: Vec<Shape>,
}

impl Default for SpecificShapeBuffers {
    fn default() -> Self {
        SpecificShapeBuffers {
            normal: Vec::new(),
            negative: Vec::new(),
            stickiness: Vec::new(),
            neg_stickiness: Vec::new(),
        }
    }
}

impl SpecificShapeBuffers {
    pub fn new() -> Self {
        SpecificShapeBuffers {
            normal: Vec::new(),
            negative: Vec::new(),
            stickiness: Vec::new(),
            neg_stickiness: Vec::new(),
        }
    }

    pub fn clear_buffers(&mut self) {
        self.normal.clear();
        self.negative.clear();
        self.stickiness.clear();
        self.neg_stickiness.clear();
    }
}
