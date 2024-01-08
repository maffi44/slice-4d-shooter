use crate::systems::{world::World, static_obj::StaticObject};

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
pub struct ShapesArrayMetadataUniform {
    pub cubes: [u32;4],
    pub spheres: [u32;4],
    pub cubes_inf_w: [u32;4],
    pub sph_cubes: [u32;4],

    pub neg_cubes: [u32;4],
    pub neg_spheres: [u32;4],
    pub neg_cubes_inf_w: [u32;4],
    pub neg_sph_cubes: [u32;4],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShapesArrayUniformData {
    pub metadata: ShapesArrayMetadataUniform,
    pub shapes: [[f32;8];512],
}

impl ShapesArrayUniformData {
    pub fn new(world: &World) -> Self {
        let mut cubes = Vec::new();
        let mut spheres = Vec::new();
        let mut inf_w_cubes = Vec::new();
        let mut sph_cubes = Vec::new();

        let mut neg_cubes = Vec::new();
        let mut neg_spheres = Vec::new();
        let mut neg_inf_w_cubes = Vec::new();
        let mut neg_sph_cubes = Vec::new();
        
        for obj in &world.map {
            match obj {
                StaticObject::Cube(tr,size, is_positive) => {
                    let mut data = [0.;8];

                    tr.get_position().write_to_slice(&mut data);
                    size.write_to_slice(&mut data[4..]);

                    if *is_positive {
                        cubes.push(data);
                    } else {
                        neg_cubes.push(data);
                    }
                }
                StaticObject::Sphere(tr,size, is_positive) => {
                    let mut data = [0.;8];

                    tr.get_position().write_to_slice(&mut data);
                    size.write_to_slice(&mut data[4..]);

                    if *is_positive {
                        spheres.push(data);
                    } else {
                        neg_spheres.push(data);
                    }
                    
                }
                StaticObject::CubeInfW(tr,size, is_positive) => {
                    let mut data = [0.;8];

                    tr.get_position().write_to_slice(&mut data);
                    size.write_to_slice(&mut data[4..]);

                    if *is_positive {
                        inf_w_cubes.push(data);
                    } else {
                        neg_inf_w_cubes.push(data);
                    }
                },
                StaticObject::SphCube(tr,size, is_positive) => {
                    let mut data = [0.;8];

                    tr.get_position().write_to_slice(&mut data);
                    size.write_to_slice(&mut data[4..]);

                    if *is_positive {
                        sph_cubes.push(data);
                    } else {
                        neg_sph_cubes.push(data);
                    }
                }
            }
        }

        let mut shapes = [[0.0_f32;8];512];

        // copy shapes to array and make metadata
        // copy cubes
        let first_index = 0;
        let amount = cubes.len();
        let last_index = first_index + amount;

        let cubes_metadata = [first_index as u32, amount as u32, 0, 0];
        for i in first_index.. first_index + amount {
            shapes[i] = cubes[i - first_index];
        }

        // copy spheres
        let first_index = last_index;
        let amount = spheres.len();
        let last_index = first_index + amount;

        let spheres_metadata = [first_index as u32, amount as u32, 0, 0];
        for i in first_index.. first_index + amount {
            shapes[i] = spheres[i - first_index];
        }

        // copy w_inf_cubes
        let first_index = last_index;
        let amount = inf_w_cubes.len();
        let last_index = first_index + amount;

        let inf_w_cubes_metadata = [first_index as u32, amount as u32, 0, 0];
        for i in first_index.. first_index + amount {
            shapes[i] = inf_w_cubes[i - first_index];
        }

        // copy shp_cubes
        let first_index = last_index;
        let amount = sph_cubes.len();
        let last_index = first_index + amount;

        let sph_cubes_metadata = [first_index as u32, amount as u32, 0, 0];
        for i in first_index.. first_index + amount {
            shapes[i] = sph_cubes[i - first_index];
        }

        // copy neg_cubes
        let first_index = last_index;
        let amount = neg_cubes.len();
        let last_index = first_index + amount;

        let neg_cubes_metadata = [first_index as u32, amount as u32, 0, 0];
        for i in first_index.. first_index + amount {
            shapes[i] = neg_cubes[i - first_index];
        }

        // copy neg_spheres
        let first_index = last_index;
        let amount = neg_spheres.len();
        let last_index = first_index + amount;

        let neg_spheres_metadata = [first_index as u32, amount as u32, 0, 0];
        for i in first_index.. first_index + amount {
            shapes[i] = neg_spheres[i - first_index];
        }

        // copy neg_w_inf_cubes
        let first_index = last_index;
        let amount = neg_inf_w_cubes.len();
        let last_index = first_index + amount;

        let neg_inf_w_cubes_metadata = [first_index as u32, amount as u32, 0, 0];
        for i in first_index.. first_index + amount {
            shapes[i] = neg_inf_w_cubes[i - first_index];
        }

        // copy neg_shp_cubes
        let first_index = last_index;
        let amount = neg_sph_cubes.len();
        let last_index = first_index + amount;

        let neg_sph_cubes_metadata = [first_index as u32, amount as u32, 0, 0];
        for i in first_index.. first_index + amount {
            shapes[i] = neg_sph_cubes[i - first_index];
        }

        let metadata = ShapesArrayMetadataUniform {
            spheres: spheres_metadata,
            cubes: cubes_metadata,
            cubes_inf_w: inf_w_cubes_metadata,
            sph_cubes: sph_cubes_metadata,

            neg_spheres: neg_spheres_metadata,
            neg_cubes: neg_cubes_metadata,
            neg_cubes_inf_w: neg_inf_w_cubes_metadata,
            neg_sph_cubes: neg_sph_cubes_metadata,
        };

        ShapesArrayUniformData {
            shapes,
            metadata
        }
    }
}