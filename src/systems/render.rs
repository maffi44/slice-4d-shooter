mod renderer;

use renderer::Renderer;

use winit::window::Window;

use glam::{Vec4, Mat4};

use super::{
    world::World,
};

#[derive(Clone, Copy)]
pub struct FrameRenderData {
    cam_position: Vec4,
    cam_rotation: Mat4,

}

impl FrameRenderData {
    pub fn new(
        cam_position: Vec4,
        cam_rotation: Mat4,
    ) -> Self {
        FrameRenderData {
            cam_position,
            cam_rotation
        }
    }
}

pub struct RenderSystem {
    pub window: Window,
    renderer: Renderer,
}

impl RenderSystem {
    pub async fn new(
        window: Window,
    ) -> Self {
        let renderer = Renderer::new(&window).await;

        RenderSystem {
            window,
            renderer,
        }
    }

    pub fn render_frame(&mut self, world: &mut World) {
        
        let position;
        let rot_matrix;
        
        if let Some(main_player) = world.pool_of_players.get(&world.main_camera_from) {
            position = main_player.get_position();
            rot_matrix = main_player.get_rotation_matrix();
        } else {
            position = Vec4::ZERO;
            rot_matrix = Mat4::IDENTITY;
        }

        let render_data = FrameRenderData::new(
            position,
            rot_matrix,
        );

        if let Err(err) = self.renderer.render(render_data) {
            match err {
                wgpu::SurfaceError::Lost => self.resize_frame_buffer(),

                // The system is out of memory, we should probably qui
                wgpu::SurfaceError::OutOfMemory => panic!("Out of GPU memory"),

                // All other errors (Outdated, Timeout) should be resolved by the next frame
                _ => log::error!("{:?}", err),
            }
        }
    }

    pub fn resize_frame_buffer(&mut self) {
        self.renderer.resize(self.window.inner_size());
    }

}