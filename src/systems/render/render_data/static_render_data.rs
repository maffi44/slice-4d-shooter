use crate::systems::{
    world::World,
    physics::physics_system_data::ShapeType,
    render::render_data::{
        ShapesArraysMetadata,
        Shape
    },
};

use super::ShapesArrays;

pub struct StaticRenderData {
    pub static_shapes_data: ShapesArrays,
    pub other_static_data: OtherStaticData,
}


#[repr(C)]
#[derive(Debug, Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct OtherStaticData {
    shapes_arrays_metadata: ShapesArraysMetadata,

    empty_bytes: [f32; 3],

    static_shapes_stickiness: f32 
}

impl OtherStaticData {
    pub fn new(shapes_arrays_metadata: ShapesArraysMetadata, stickiness: f32) -> Self {
        OtherStaticData {
            shapes_arrays_metadata,

            empty_bytes: [0.0, 0.0, 0.0],

            static_shapes_stickiness: stickiness
        }
    }
}

impl StaticRenderData {
    pub fn new(world: &World) -> Self {

        let mut shapes = ShapesArrays::default();

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

        let mut cubes: Vec<Shape> = Vec::new();
        let mut s_cubes: Vec<Shape> = Vec::new();
        let mut neg_cubes: Vec<Shape> = Vec::new();
        let mut s_neg_cubes: Vec<Shape> = Vec::new();

        let mut spheres: Vec<Shape> = Vec::new();
        let mut s_spheres: Vec<Shape> = Vec::new();
        let mut neg_spheres: Vec<Shape> = Vec::new();
        let mut s_neg_spheres: Vec<Shape> = Vec::new();

        let mut sph_cubes: Vec<Shape> = Vec::new();
        let mut s_sph_cubes: Vec<Shape> = Vec::new();
        let mut neg_sph_cubes: Vec<Shape> = Vec::new();
        let mut s_neg_sph_cubes: Vec<Shape> = Vec::new();

        let mut inf_w_cubes: Vec<Shape> = Vec::new();
        let mut s_inf_w_cubes: Vec<Shape> = Vec::new();
        let mut neg_inf_w_cubes: Vec<Shape> = Vec::new();
        let mut s_neg_inf_w_cubes: Vec<Shape> = Vec::new();

        for obj in &world.level.static_objects {

            log::info!("static objects amount is {}", world.level.static_objects.len());

            match obj.collider.shape_type {
                ShapeType::Cube => {

                    if obj.collider.is_positive {
                        if !obj.collider.stickiness {

                            let shape = Shape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                roundness: obj.collider.roundness,
                            };

                            cubes.push(shape);

                            cubes_amount += 1;

                        } else {

                            let shape = Shape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                roundness: obj.collider.roundness,
                            };

                            s_cubes.push(shape);

                            s_cubes_amount += 1;
                        }
                    } else {
                        if !obj.collider.stickiness {
                            let shape = Shape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                roundness: obj.collider.roundness,
                            };

                            neg_cubes.push(shape);

                            neg_cubes_amount += 1;
                        } else {
                            let shape = Shape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                roundness: obj.collider.roundness,
                            };

                            s_neg_cubes.push(shape);

                            s_neg_cubes_amount += 1;
                        }
                    }
                }
                ShapeType::Sphere => {
                    if obj.collider.is_positive {
                        if !obj.collider.stickiness {

                            let shape = Shape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                roundness: obj.collider.roundness,
                            };

                            spheres.push(shape);

                        } else {

                            let shape = Shape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                roundness: obj.collider.roundness,
                            };

                            s_spheres.push(shape);
                        }
                    } else {
                        if !obj.collider.stickiness {
                            let shape = Shape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                roundness: obj.collider.roundness,
                            };

                            neg_spheres.push(shape);
                        } else {
                            let shape = Shape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                roundness: obj.collider.roundness,
                            };

                            s_neg_spheres.push(shape);
                        }
                    }
                }
                ShapeType::CubeInfW => {
                    if obj.collider.is_positive {
                        if !obj.collider.stickiness {

                            let shape = Shape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                roundness: obj.collider.roundness,
                            };

                            inf_w_cubes.push(shape);

                        } else {

                            let shape = Shape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                roundness: obj.collider.roundness,
                            };

                            s_inf_w_cubes.push(shape);
                        }
                    } else {
                        if !obj.collider.stickiness {
                            let shape = Shape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                roundness: obj.collider.roundness,
                            };

                            neg_inf_w_cubes.push(shape);
                        } else {
                            let shape = Shape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                roundness: obj.collider.roundness,
                            };

                            s_neg_inf_w_cubes.push(shape);
                        }
                    }
                },
                ShapeType::SphCube => {
                    if obj.collider.is_positive {
                        if !obj.collider.stickiness {

                            let shape = Shape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                roundness: obj.collider.roundness,
                            };

                            sph_cubes.push(shape);

                        } else {

                            let shape = Shape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                roundness: obj.collider.roundness,
                            };

                            s_sph_cubes.push(shape);
                        }
                    } else {
                        if !obj.collider.stickiness {
                            let shape = Shape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                roundness: obj.collider.roundness,
                            };

                            neg_sph_cubes.push(shape);
                        } else {
                            let shape = Shape {
                                pos: obj.collider.position.to_array(),
                                size: obj.collider.size.to_array(),
                                color: obj.material.color.to_array(),
                                roundness: obj.collider.roundness,
                            };

                            s_neg_sph_cubes.push(shape);
                        }
                    }
                }
            }
        }


        // packing normal shapes
        let mut index = 0;
        cubes_start = 0u32;

        for shape in cubes {
            shapes.normal[index] = shape;
            index += 1;
        }

        cubes_amount = index as u32;


        spheres_start = index as u32;

        for shape in spheres {
            shapes.normal[index] = shape;
            index += 1;
        }

        spheres_amount = index as u32 - spheres_start;


        inf_cubes_start = index as u32;

        for shape in inf_w_cubes {
            shapes.normal[index] = shape;
            index += 1;
        }

        inf_cubes_amount = index as u32 - inf_cubes_start;


        sph_cubes_start = index as u32;

        for shape in sph_cubes {
            shapes.normal[index] = shape;
            index += 1;
        }

        sph_cubes_amount = index as u32 - sph_cubes_start;


        // packing stickiness shapes
        let mut index = 0;
        s_cubes_start = 0u32;

        for shape in s_cubes {
            shapes.stickiness[index] = shape;
            index += 1;
        }

        s_cubes_amount = index as u32;


        s_spheres_start = index as u32;

        for shape in s_spheres {
            shapes.stickiness[index] = shape;
            index += 1;
        }

        s_spheres_amount = index as u32 - s_spheres_start;


        s_inf_cubes_start = index as u32;

        for shape in s_inf_w_cubes {
            shapes.stickiness[index] = shape;
            index += 1;
        }

        s_inf_cubes_amount = index as u32 - s_inf_cubes_start;


        s_sph_cubes_start = index as u32;

        for shape in s_sph_cubes {
            shapes.stickiness[index] = shape;
            index += 1;
        }

        s_sph_cubes_amount = index as u32 - s_sph_cubes_start;



        // packing negative shapes
        let mut index = 0;
        neg_cubes_start = 0u32;

        for shape in neg_cubes {
            shapes.negative[index] = shape;
            index += 1;
        }

        neg_cubes_amount = index as u32;


        neg_spheres_start = index as u32;

        for shape in neg_spheres {
            shapes.negative[index] = shape;
            index += 1;
        }

        neg_spheres_amount = index as u32 - neg_spheres_start;


        neg_inf_cubes_start = index as u32;

        for shape in neg_inf_w_cubes {
            shapes.negative[index] = shape;
            index += 1;
        }

        neg_inf_cubes_amount = index as u32 - neg_inf_cubes_start;


        neg_sph_cubes_start = index as u32;

        for shape in neg_sph_cubes {
            shapes.negative[index] = shape;
            index += 1;
        }

        neg_sph_cubes_amount = index as u32 - neg_sph_cubes_start;



        // packing negative and stickiness shapes
        let mut index = 0;
        s_neg_cubes_start = 0u32;

        for shape in s_neg_cubes {
            shapes.neg_stickiness[index] = shape;
            index += 1;
        }

        s_neg_cubes_amount = index as u32;


        s_neg_spheres_start = index as u32;

        for shape in s_neg_spheres {
            shapes.neg_stickiness[index] = shape;
            index += 1;
        }

        s_neg_spheres_amount = index as u32 - s_neg_spheres_start;


        s_neg_inf_cubes_start = index as u32;

        for shape in s_neg_inf_w_cubes {
            shapes.neg_stickiness[index] = shape;
            index += 1;
        }

        s_neg_inf_cubes_amount = index as u32 - s_neg_inf_cubes_start;


        s_neg_sph_cubes_start = index as u32;

        for shape in s_neg_sph_cubes {
            shapes.neg_stickiness[index] = shape;
            index += 1;
        }

        s_neg_sph_cubes_amount = index as u32 - s_neg_sph_cubes_start;






        let metadata = ShapesArraysMetadata {
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
        
        log::info!("static shapes metadata: \n{:?}", metadata);
        
        let other_static_data = OtherStaticData::new(
            metadata,
            world.level.all_shapes_stickiness_radius
        );

        StaticRenderData {
            static_shapes_data: shapes,
            other_static_data,
        }
    }
}