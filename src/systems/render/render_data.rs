use crate::systems::{
    physics::physics_system_data::ShapeType,
    world::{
        static_object::StaticObject,
        World,
    }
};

pub struct FrameRenderData {
    pub camera_uniform: CameraUniform,
    pub time: TimeUniform,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TimeUniform {
    pub time: [f32; 4],
}

impl TimeUniform {
    pub fn new_zero() -> Self {
        TimeUniform {
            time: [
                0.0, 0.0, 0.0, 0.0
            ]
        }
    }

    pub fn new_val(val: f32) -> Self {
        TimeUniform {
            time: [
                val, 0.0, 0.0, 0.0
            ]
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub cam_pos: [f32; 4],
    pub cam_rot: [f32; 16],
    // in shader aspect is f32 but I need to add 3 zero bytes
    // to align struct layout in GPU memroy
    pub aspect: [f32; 4], 
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Shape {
    pub pos: [f32;4],
    pub size: [f32;4],
    pub color: [f32;3],
    pub roundness: f32,
}

impl Default for Shape {
    fn default() -> Self {
        Shape {
            pos: [0.0, 0.0, 0.0, 0.0],
            size: [1.0, 1.0, 1.0, 1.0],
            color: [1.0, 1.0, 1.0],
            roundness: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct NegShape {
    pub pos: [f32;4],
    pub size: [f32;4],
    pub roundness: f32,
    pub empty_bytes: [f32;3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct StickinessNegShape {
    pub pos: [f32;4],
    pub size: [f32;4],
    pub roundness: f32,
    pub stickiness: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct StickinessShape {
    pub pos: [f32;4],
    pub size: [f32;4],
    pub color: [f32;3],
    pub roundness: f32,
    pub stickiness: f32,
}

pub struct ShapeArrays {
    pub normal: [Shape; 256],
    pub negative: [NegShape; 256],
    pub stickiness: [StickinessShape; 256],
    pub neg_stickiness: [StickinessNegShape; 256],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AllShapesArraysMetadata {
    cubes_amount: u32,
    s_cubes_amount: u32,
    neg_cubes_amount: u32,
    s_neg_cubes_amount: u32,

    spheres_amount: u32,
    s_spheres_amount: u32,
    neg_spheres_amount: u32,
    s_neg_spheres_amount: u32,

    inf_w_cubes_amount: u32,
    s_inf_w_cubes_amount: u32,
    neg_inf_w_cubes_amount: u32,
    s_neg_inf_w_cubes_amount: u32,

    sph_cubes_amount: u32,
    s_sph_cubes_amount: u32,
    neg_sph_cubes_amount: u32,
    s_neg_sph_cubes_amount: u32,
}

pub struct StaticShapesArraysUniformData {
    pub cubes: ShapeArrays,
    pub spheres: ShapeArrays,
    pub inf_w_cubes: ShapeArrays,
    pub sph_cubes: ShapeArrays,

    pub metadata: AllShapesArraysMetadata,
}

impl StaticShapesArraysUniformData {
    pub fn new(world: &World) -> Self {
        let mut cubes = ShapeArrays {
            normal: [Shape::default(); 256],
            negative: [NegShape::default(); 256],
            stickiness: [StickinessShape::default(); 256],
            neg_stickiness: [StickinessNegShape::default(); 256],
        };

        let mut spheres = ShapeArrays {
            normal: [Shape::default(); 256],
            negative: [NegShape::default(); 256],
            stickiness: [StickinessShape::default(); 256],
            neg_stickiness: [StickinessNegShape::default(); 256],
        };

        let mut inf_w_cubes = ShapeArrays {
            normal: [Shape::default(); 256],
            negative: [NegShape::default(); 256],
            stickiness: [StickinessShape::default(); 256],
            neg_stickiness: [StickinessNegShape::default(); 256],
        };

        let mut sph_cubes = ShapeArrays {
            normal: [Shape::default(); 256],
            negative: [NegShape::default(); 256],
            stickiness: [StickinessShape::default(); 256],
            neg_stickiness: [StickinessNegShape::default(); 256],
        };
        
        let mut cubes_amount = 0u32;
        let mut spheres_amount = 0u32;
        let mut sph_cubes_amount = 0u32;
        let mut inf_w_cubes_amount = 0u32;

        let mut neg_cubes_amount = 0u32;
        let mut neg_spheres_amount = 0u32;
        let mut neg_sph_cubes_amount = 0u32;
        let mut neg_inf_w_cubes_amount = 0u32;

        let mut s_cubes_amount = 0u32;
        let mut s_spheres_amount = 0u32;
        let mut s_sph_cubes_amount = 0u32;
        let mut s_inf_w_cubes_amount = 0u32;

        let mut s_neg_cubes_amount = 0u32;
        let mut s_neg_spheres_amount = 0u32;
        let mut s_neg_sph_cubes_amount = 0u32;
        let mut s_neg_inf_w_cubes_amount = 0u32;

        for obj in &world.level.static_objects {

            match obj.collider.shape_type {
                ShapeType::Cube => {

                    if obj.collider.is_positive {
                        if obj.collider.stickiness == 0.0 {

                            let shape = Shape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                roundness: obj.collider.roundness,
                            };

                            // let shape = Shape {
                            //     pos: [0.0, 0.0, 0.0, 0.0],
                            //     size: [1.0, 1.0, 1.0, 1.0],
                            //     color: [1.0, 1.0, 1.0],
                            //     roundness: 0.0,
                            // };

                            cubes.normal[cubes_amount as usize] = shape;

                            cubes_amount += 1;

                        } else {

                            let shape = StickinessShape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                stickiness: obj.collider.stickiness,
                                roundness: obj.collider.roundness,
                            };

                            cubes.stickiness[s_cubes_amount as usize] = shape;

                            s_cubes_amount += 1;
                        }
                    } else {
                        if obj.collider.stickiness == 0.0 {
                            let shape = NegShape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                roundness: obj.collider.roundness,
                                empty_bytes: [0.0, 0.0, 0.0]
                            };

                            cubes.negative[neg_cubes_amount as usize] = shape;

                            neg_cubes_amount += 1;
                        } else {
                            let shape = StickinessNegShape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                roundness: obj.collider.roundness,
                                stickiness: obj.collider.stickiness
                            };

                            cubes.neg_stickiness[s_neg_cubes_amount as usize] = shape;

                            s_neg_cubes_amount += 1;
                        }
                    }
                }
                ShapeType::Sphere => {
                    if obj.collider.is_positive {
                        if obj.collider.stickiness == 0.0 {

                            let shape = Shape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                roundness: obj.collider.roundness,
                            };

                            spheres.normal[spheres_amount as usize] = shape;

                            spheres_amount += 1;

                        } else {

                            let shape = StickinessShape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                stickiness: obj.collider.stickiness,
                                roundness: obj.collider.roundness,
                            };

                            spheres.stickiness[s_spheres_amount as usize] = shape;

                            s_spheres_amount += 1;
                        }
                    } else {
                        if obj.collider.stickiness == 0.0 {
                            let shape = NegShape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                roundness: obj.collider.roundness,
                                empty_bytes: [0.0, 0.0, 0.0]
                            };

                            spheres.negative[neg_spheres_amount as usize] = shape;

                            neg_spheres_amount += 1;
                        } else {
                            let shape = StickinessNegShape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                roundness: obj.collider.roundness,
                                stickiness: obj.collider.stickiness
                            };

                            spheres.neg_stickiness[s_neg_spheres_amount as usize] = shape;

                            s_neg_spheres_amount += 1;
                        }
                    }
                }
                ShapeType::CubeInfW => {
                    if obj.collider.is_positive {
                        if obj.collider.stickiness == 0.0 {

                            let shape = Shape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                roundness: obj.collider.roundness,
                            };

                            inf_w_cubes.normal[inf_w_cubes_amount as usize] = shape;

                            inf_w_cubes_amount += 1;

                        } else {

                            let shape = StickinessShape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                stickiness: obj.collider.stickiness,
                                roundness: obj.collider.roundness,
                            };

                            inf_w_cubes.stickiness[s_inf_w_cubes_amount as usize] = shape;

                            s_inf_w_cubes_amount += 1;
                        }
                    } else {
                        if obj.collider.stickiness == 0.0 {
                            let shape = NegShape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                roundness: obj.collider.roundness,
                                empty_bytes: [0.0, 0.0, 0.0]
                            };

                            inf_w_cubes.negative[neg_inf_w_cubes_amount as usize] = shape;

                            neg_inf_w_cubes_amount += 1;
                        } else {
                            let shape = StickinessNegShape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                roundness: obj.collider.roundness,
                                stickiness: obj.collider.stickiness
                            };

                            inf_w_cubes.neg_stickiness[s_neg_inf_w_cubes_amount as usize] = shape;

                            s_neg_inf_w_cubes_amount += 1;
                        }
                    }
                },
                ShapeType::SphCube => {
                    if obj.collider.is_positive {
                        if obj.collider.stickiness == 0.0 {

                            let shape = Shape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                roundness: obj.collider.roundness,
                            };

                            sph_cubes.normal[sph_cubes_amount as usize] = shape;

                            sph_cubes_amount += 1;

                        } else {

                            let shape = StickinessShape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                stickiness: obj.collider.stickiness,
                                roundness: obj.collider.roundness,
                            };

                            sph_cubes.stickiness[s_sph_cubes_amount as usize] = shape;

                            s_sph_cubes_amount += 1;
                        }
                    } else {
                        if obj.collider.stickiness == 0.0 {
                            let shape = NegShape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                roundness: obj.collider.roundness,
                                empty_bytes: [0.0, 0.0, 0.0]
                            };

                            sph_cubes.negative[neg_sph_cubes_amount as usize] = shape;

                            neg_sph_cubes_amount += 1;
                        } else {
                            let shape = StickinessNegShape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                roundness: obj.collider.roundness,
                                stickiness: obj.collider.stickiness
                            };

                            sph_cubes.neg_stickiness[s_neg_sph_cubes_amount as usize] = shape;

                            s_neg_sph_cubes_amount += 1;
                        }
                    }
                }
            }
        }

        let metadata = AllShapesArraysMetadata {
            cubes_amount,
            spheres_amount,
            sph_cubes_amount,
            inf_w_cubes_amount,
            neg_cubes_amount,
            neg_spheres_amount,
            neg_sph_cubes_amount,
            neg_inf_w_cubes_amount,
            s_cubes_amount,
            s_spheres_amount,
            s_sph_cubes_amount,
            s_inf_w_cubes_amount,
            s_neg_cubes_amount,
            s_neg_spheres_amount,
            s_neg_sph_cubes_amount,
            s_neg_inf_w_cubes_amount,
        };

        StaticShapesArraysUniformData {
            cubes,
            sph_cubes,
            spheres,
            inf_w_cubes,
            metadata,
        }
    }
}