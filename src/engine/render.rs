pub mod render_data;
mod renderer;

use std::sync::{Arc, Mutex};

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

#[cfg(not(target_arch="wasm32"))]
use tokio::{runtime::Runtime, time::sleep};
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
    renderer: Arc<Mutex<Renderer>>,
}



impl RenderSystem {
    pub async fn new(
        window: Window,
        world: &World,
        time: &TimeSystem,
        #[cfg(not(target_arch="wasm32"))]
        runtime: &mut Runtime,
    ) -> Self {
        
        let render_data = RenderData::new(world, time, &window);
        
        let renderer = Arc::new(
            Mutex::new(
                Renderer::new(&window, &render_data, time.target_frame_duration.as_secs_f64()).await
            )
        );

        // spawn async tusk for renderer
        let async_renderer = renderer.clone();
        #[cfg(not(target_arch="wasm32"))]
        runtime.spawn(async move {
            loop {

                match async_renderer.try_lock() {
                    Ok(mut renderer_lock) => {
                        if let Err(err) = renderer_lock.render(/*&self.window*/) {
                            match err {
                                // wgpu::SurfaceError::Lost => renderer_lock.resize(self.window.inner_size()),
                
                                // The system is out of memory, we should probably quit
                                wgpu::SurfaceError::OutOfMemory => panic!("Out of GPU memory"),
                
                                // All other errors (Outdated, Timeout) should be resolved by the next frame
                                _ => log::error!("{:?}", err),
                            }
                        }
                    }
                    Err(_) => {}
                }

                tokio::time::sleep(tokio::time::Duration::from_micros(500)).await;
            }
        });


        log::info!("render system: renderer init");
        
        RenderSystem {
            window,
            renderer,

            render_data,
        }
    }



    pub fn send_data_to_renderer(&mut self, world: &World, time: &TimeSystem) {

        self.render_data.update_dynamic_render_data(
            world,
            time,
            &self.window,
            &self.render_data.static_data.static_bounding_box.clone()
        );

        let mut renderer_lock = self.renderer.lock().unwrap();

        renderer_lock.queue.write_buffer(
            &renderer_lock.other_dynamic_data_buffer,
            0,
            bytemuck::cast_slice(&[self.render_data.dynamic_data.other_dynamic_data]),
        );

        renderer_lock.queue.write_buffer(
            &renderer_lock.dynamic_normal_shapes_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.dynamic_shapes_data.normal.as_slice()),
        );

        renderer_lock.queue.write_buffer(
            &renderer_lock.dynamic_negative_shapes_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.dynamic_shapes_data.negative.as_slice()),
        );

        renderer_lock.queue.write_buffer(
            &renderer_lock.dynamic_stickiness_shapes_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.dynamic_shapes_data.stickiness.as_slice()),
        );

        renderer_lock.queue.write_buffer(
            &renderer_lock.dynamic_neg_stickiness_shapes_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.dynamic_shapes_data.neg_stickiness.as_slice()),
        );

        renderer_lock.queue.write_buffer(
            &renderer_lock.spherical_areas_data_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.spherical_areas_data.as_slice()),
        );

        renderer_lock.queue.write_buffer(
            &renderer_lock.beam_areas_data_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.beam_areas_data.as_slice()),
        );

        renderer_lock.queue.write_buffer(
            &renderer_lock.player_forms_data_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.player_forms_data.as_slice()),
        );

        #[cfg(target_arch="wasm32")]
        if let Err(err) = renderer_lock.render(/*&self.window*/) {
            match err {
                wgpu::SurfaceError::Lost => renderer_lock.resize(self.window.inner_size()),

                // The system is out of memory, we should probably quit
                wgpu::SurfaceError::OutOfMemory => panic!("Out of GPU memory"),

                // All other errors (Outdated, Timeout) should be resolved by the next frame
                _ => log::error!("{:?}", err),
            }
        }
        

        
    }



    pub fn resize_frame_buffer(&mut self) {
        self.renderer
            .lock()
            .unwrap()
            .resize(self.window.inner_size());
    }
}

