pub mod render_data;
mod renderer;

use crate::systems::render::render_data::TimeUniform;

use self::render_data::CameraUniform;

use super::{
    world::World,
    time::TimeSystem
};

use renderer::Renderer;
use render_data::FrameRenderData;
use winit::window::Window;
use glam::{Vec4, Mat4};

pub struct RenderSystem {
    pub window: Window,
    renderer: Renderer,
}

impl RenderSystem {
    pub async fn new(
        window: Window,
        world: &World
    ) -> Self {


        let renderer = Renderer::new(&window, world).await;

        RenderSystem {
            window,
            renderer,
        }
    }

    pub fn render_frame(&mut self, world: &mut World, time: &mut TimeSystem) {
        
        let cam_pos;
        let cam_rot;
        
        if let Some(main_player) = world.pool_of_players.get(&world.main_camera_from) {
            cam_pos = main_player.get_position();
            cam_rot = main_player.get_rotation_matrix();
        } else {
            cam_pos = Vec4::ZERO;
            cam_rot = Mat4::IDENTITY;
        }

        let aspect = {
            let size = winit::dpi::PhysicalSize::new(1200, 800);
            (size.width / size.height) as f32
        };

        let mut rot_mat_slice: [f32;16] = [0.0; 16];

        cam_rot.write_cols_to_slice(&mut rot_mat_slice);

        let render_data = FrameRenderData {
            camera_uniform: CameraUniform {
                cam_pos: cam_pos.into(),
                cam_rot: rot_mat_slice,
                aspect: [aspect, 0.0, 0.0, 0.0],
            },
            time: TimeUniform::new_val(time.timestamp_of_main_loop_start.elapsed().as_secs_f32()),
        };

        self.renderer.queue.write_buffer(
            &self.renderer.camera_buffer,
            0,
            bytemuck::cast_slice(&[render_data.camera_uniform]),
        );

        self.renderer.queue.write_buffer(
            &self.renderer.time_buffer,
            0,
            bytemuck::cast_slice(&[render_data.time]),
        );

        if let Err(err) = self.renderer.render() {
            match err {
                wgpu::SurfaceError::Lost => self.resize_frame_buffer(),

                // The system is out of memory, we should probably quit
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