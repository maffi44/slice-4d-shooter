pub struct FrameRenderData {
    pub camera_uniform: CameraUniform,
    pub time: TimeUniform,
}

// impl FrameRenderData {
//     pub fn new(
//         camera_uniform: CameraUniform,
//         time: f32,
//     ) -> Self {
//         FrameRenderData {
//             camera_uniform,
//             time
//         }
//     }
// }

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
    pub spheres: [u32;2],
    pub cubes: [u32;2],
    pub cubes_inf_w: [u32;2],
    
    pub empty_bytes_for_aligment: [u32;2]
}

impl ShapesArrayMetadataUniform {
    pub fn new_zero() -> Self {
        ShapesArrayMetadataUniform {
            spheres: [0,0],
            cubes: [0,0],
            cubes_inf_w: [0,0],
            empty_bytes_for_aligment: [0,0],
        }
    }
}