use glam::{Mat4, Vec4};
use winit::window::Window;

use crate::systems::{
    physics::physics_system_data::ShapeType,
    time::TimeSystem,
    world::World,
    actor::{
        Actor,
        ActorWrapper
    },
};

use super::{
    Shape,
    ShapesArrays,
    ShapesArraysMetadata
};

pub struct DynamicRenderData {
    pub dynamic_shapes_data: ShapesArrays,
    pub other_dynamic_data: OtherDynamicData,

    // frame memory buffers
    cubes_buffer: SpecificShapeBuffers,
    spheres_buffer: SpecificShapeBuffers,
    sph_cubes_buffer: SpecificShapeBuffers,
    inf_w_cubes_buffer: SpecificShapeBuffers,
}

impl DynamicRenderData {
    pub fn new(world: &World, time: &TimeSystem, window: &Window) -> Self {
        let mut dynamic_render_data = DynamicRenderData {
            dynamic_shapes_data: ShapesArrays::default(),
            other_dynamic_data: OtherDynamicData::default(),

            cubes_buffer: SpecificShapeBuffers::default(),
            spheres_buffer: SpecificShapeBuffers::default(),
            sph_cubes_buffer: SpecificShapeBuffers::default(),
            inf_w_cubes_buffer: SpecificShapeBuffers::default(),
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
        
        self.cubes_buffer.clear_buffers();
        self.spheres_buffer.clear_buffers();
        self.sph_cubes_buffer.clear_buffers();
        self.inf_w_cubes_buffer.clear_buffers();


        for (_, actor) in world.actors.iter() {

            if let Some(visual_element) = actor.get_visual_element() {

                let transform = visual_element.transfrom;

                for static_object in visual_element.static_objects {
                    
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
                                    self.cubes_buffer.stickiness.push(shape);
                                } else {
                                    self.cubes_buffer.normal.push(shape);
                                }
                            } else {
                                if is_stickiness {
                                    self.cubes_buffer.neg_stickiness.push(shape);
                                } else {
                                    self.cubes_buffer.negative.push(shape);
                                }
                            }
                        },
                        ShapeType::Sphere => {
                            if is_positive {
                                if is_stickiness {
                                    self.spheres_buffer.stickiness.push(shape);
                                } else {
                                    self.spheres_buffer.normal.push(shape);
                                }
                            } else {
                                if is_stickiness {
                                    self.spheres_buffer.neg_stickiness.push(shape);
                                } else {
                                    self.spheres_buffer.negative.push(shape);
                                }
                            }
                            
                        },
                        ShapeType::SphCube => {
                            if is_positive {
                                if is_stickiness {
                                    self.sph_cubes_buffer.stickiness.push(shape);
                                } else {
                                    self.sph_cubes_buffer.normal.push(shape);
                                }
                            } else {
                                if is_stickiness {
                                    self.sph_cubes_buffer.neg_stickiness.push(shape);
                                } else {
                                    self.sph_cubes_buffer.negative.push(shape);
                                }
                            }
                            
                        },
                        ShapeType::CubeInfW => {
                            if is_positive {
                                if is_stickiness {
                                    self.inf_w_cubes_buffer.stickiness.push(shape);
                                } else {
                                    self.inf_w_cubes_buffer.normal.push(shape);
                                }
                            } else {
                                if is_stickiness {
                                    self.inf_w_cubes_buffer.neg_stickiness.push(shape);
                                } else {
                                    self.inf_w_cubes_buffer.negative.push(shape);
                                }
                            }
                        }
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

        while let Some(shape) = self.cubes_buffer.normal.pop() {
            self.dynamic_shapes_data.normal[index] = shape;
            index += 1;
        }

        cubes_amount = index as u32;


        spheres_start = index as u32;

       while let Some(shape) = self.spheres_buffer.normal.pop() {
            self.dynamic_shapes_data.normal[index] = shape;
            index += 1;
        }

        spheres_amount = index as u32 - spheres_start;


        inf_cubes_start = index as u32;

       while let Some(shape) = self.inf_w_cubes_buffer.normal.pop() {
            self.dynamic_shapes_data.normal[index] = shape;
            index += 1;
        }

        inf_cubes_amount = index as u32 - inf_cubes_start;


        sph_cubes_start = index as u32;

       while let Some(shape) = self.sph_cubes_buffer.normal.pop() {
            self.dynamic_shapes_data.normal[index] = shape;
            index += 1;
        }

        sph_cubes_amount = index as u32 - sph_cubes_start;


        // packing stickiness shapes
        let mut index = 0;
        s_cubes_start = 0u32;

       while let Some(shape) = self.cubes_buffer.stickiness.pop() {
            self.dynamic_shapes_data.stickiness[index] = shape;
            index += 1;
        }

        s_cubes_amount = index as u32;


        s_spheres_start = index as u32;

       while let Some(shape) = self.spheres_buffer.stickiness.pop() {
            self.dynamic_shapes_data.stickiness[index] = shape;
            index += 1;
        }

        s_spheres_amount = index as u32 - s_spheres_start;


        s_inf_cubes_start = index as u32;

       while let Some(shape) = self.inf_w_cubes_buffer.stickiness.pop() {
            self.dynamic_shapes_data.stickiness[index] = shape;
            index += 1;
        }

        s_inf_cubes_amount = index as u32 - s_inf_cubes_start;


        s_sph_cubes_start = index as u32;

       while let Some(shape) = self.sph_cubes_buffer.stickiness.pop() {
            self.dynamic_shapes_data.stickiness[index] = shape;
            index += 1;
        }

        s_sph_cubes_amount = index as u32 - s_sph_cubes_start;



        // packing negative shapes
        let mut index = 0;
        neg_cubes_start = 0u32;

       while let Some(shape) = self.cubes_buffer.negative.pop() {
            self.dynamic_shapes_data.negative[index] = shape;
            index += 1;
        }

        neg_cubes_amount = index as u32;


        neg_spheres_start = index as u32;

       while let Some(shape) = self.spheres_buffer.negative.pop() {
            self.dynamic_shapes_data.negative[index] = shape;
            index += 1;
        }

        neg_spheres_amount = index as u32 - neg_spheres_start;


        neg_inf_cubes_start = index as u32;

       while let Some(shape) = self.inf_w_cubes_buffer.negative.pop() {
            self.dynamic_shapes_data.negative[index] = shape;
            index += 1;
        }

        neg_inf_cubes_amount = index as u32 - neg_inf_cubes_start;


        neg_sph_cubes_start = index as u32;

       while let Some(shape) = self.sph_cubes_buffer.negative.pop() {
            self.dynamic_shapes_data.negative[index] = shape;
            index += 1;
        }

        neg_sph_cubes_amount = index as u32 - neg_sph_cubes_start;



        // packing negative and stickiness shapes
        let mut index = 0;
        s_neg_cubes_start = 0u32;

       while let Some(shape) = self.cubes_buffer.neg_stickiness.pop() {
            self.dynamic_shapes_data.neg_stickiness[index] = shape;
            index += 1;
        }

        s_neg_cubes_amount = index as u32;


        s_neg_spheres_start = index as u32;

       while let Some(shape) = self.spheres_buffer.neg_stickiness.pop() {
            self.dynamic_shapes_data.neg_stickiness[index] = shape;
            index += 1;
        }

        s_neg_spheres_amount = index as u32 - s_neg_spheres_start;


        s_neg_inf_cubes_start = index as u32;

       while let Some(shape) = self.inf_w_cubes_buffer.neg_stickiness.pop() {
            self.dynamic_shapes_data.neg_stickiness[index] = shape;
            index += 1;
        }

        s_neg_inf_cubes_amount = index as u32 - s_neg_inf_cubes_start;


        s_neg_sph_cubes_start = index as u32;

       while let Some(shape) = self.sph_cubes_buffer.neg_stickiness.pop() {
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

        self.other_dynamic_data.update(world, time, window, shapes_arrays_metadata);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct OtherDynamicData {
    dynamic_shapes_arrays_metadata: ShapesArraysMetadata,
    camera_data: CameraUniform,
    empty_byte: f32,
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
        shapes_arrays_metadata: ShapesArraysMetadata
    ) {
        
        let cam_pos;
        let cam_rot;
        
        if let Some(actor) = world.actors.get(&world.main_camera_from) {
            if let ActorWrapper::Player(main_player) = actor {
                cam_pos = main_player.get_position() + Vec4::Y * main_player.get_collider_radius() * 0.98;
                cam_rot = main_player.get_rotation_matrix();
            } else {
                panic!("main camera is connected to the actor that is not a Player")
            }
        } else {
            panic!("main camera is not connected to the player")
        }

        self.camera_data = CameraUniform {
            cam_pos: cam_pos.to_array(),
            cam_rot: cam_rot.to_cols_array(),
        };

        self.dynamic_shapes_arrays_metadata = shapes_arrays_metadata;

        self.screen_aspect = {
            let size = window.inner_size();
            size.width as f32 / size.height as f32
        };

        self.time = time.timestamp_of_main_loop_start.elapsed().as_secs_f32();
    }
}

impl Default for OtherDynamicData {
    fn default() -> Self {

        OtherDynamicData {
            dynamic_shapes_arrays_metadata: ShapesArraysMetadata::default(),
            camera_data: CameraUniform::default(),
            empty_byte: 0.0,
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
