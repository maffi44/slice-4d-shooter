use::glam::{
    Vec4, Mat4
};

pub struct FrameRenderData {
    pub camera_uniform: CameraUniform,
    pub time: f32

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
pub struct CameraUniform {
    pub cam_pos: [f32; 4],
    pub cam_rot: [f32; 16],
    // in shader aspext it is f32 but I need to add 3 zero bytes
    // to align struct layout in GPU memroy
    pub aspect: [f32; 4], 
}