pub mod render_data;
mod renderer;

use crate::{
    transform::Transform,
    engine::{
        time::TimeSystem,
        world::{
            static_object::{
                StaticObject,
                VolumeArea,
                ColoringArea,
            },
            World
        },
    }
};

use self::{
    renderer::Renderer,
    render_data::RenderData,
};

use winit::window::Window;

use super::physics::dynamic_collider::PlayersDollCollider;



pub struct VisualElement<'a> {
    pub transform: &'a Transform,
    pub static_objects: Option<&'a Vec<StaticObject>>,
    pub coloring_areas: Option<&'a Vec<ColoringArea>>,
    pub volume_areas: Option<&'a Vec<VolumeArea>>,
    pub player: Option<&'a PlayersDollCollider>,
}



pub struct RenderSystem {
    render_data: RenderData,
    pub window: Window,
    renderer: Renderer,
}



impl RenderSystem {
    pub async fn new(
        window: Window,
        world: &World,
        time: &TimeSystem,
    ) -> Self {
        
        let render_data = RenderData::new(world, time, &window);
        
        let renderer = Renderer::new(&window, &render_data).await;
        
        log::info!("render system: renderer init");
        
        RenderSystem {
            window,
            renderer,

            render_data,
        }
    }



    pub fn render_frame(&mut self, world: &World, time: &TimeSystem) {

        self.render_data.update_dynamic_render_data(world, time, &self.window);

        self.renderer.queue.write_buffer(
            &self.renderer.other_dynamic_data_buffer,
            0,
            bytemuck::cast_slice(&[self.render_data.dynamic_data.other_dynamic_data]),
        );

        self.renderer.queue.write_buffer(
            &self.renderer.dynamic_normal_shapes_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.dynamic_shapes_data.normal.as_slice()),
        );

        self.renderer.queue.write_buffer(
            &self.renderer.dynamic_negative_shapes_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.dynamic_shapes_data.negative.as_slice()),
        );

        self.renderer.queue.write_buffer(
            &self.renderer.dynamic_stickiness_shapes_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.dynamic_shapes_data.stickiness.as_slice()),
        );

        self.renderer.queue.write_buffer(
            &self.renderer.dynamic_neg_stickiness_shapes_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.dynamic_shapes_data.neg_stickiness.as_slice()),
        );

        self.renderer.queue.write_buffer(
            &self.renderer.spherical_areas_data_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.spherical_areas_data.as_slice()),
        );

        self.renderer.queue.write_buffer(
            &self.renderer.beam_areas_data_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.beam_areas_data.as_slice()),
        );

        self.renderer.queue.write_buffer(
            &self.renderer.player_forms_data_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.player_forms_data.as_slice()),
        );
        

        if let Err(err) = self.renderer.render(&self.window) {
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

