pub mod render_data;
pub mod camera;
pub mod raymarch_shader_generator;
mod renderer;
mod ui_renderer;

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

use client_server_protocol::Team;
#[cfg(not(target_arch="wasm32"))]
use tokio::runtime::Runtime;
use wgpu::Backend;
use winit::{monitor::{MonitorHandle, VideoModeHandle}, window::Window};

use super::{input::ActionsFrameState, physics::dynamic_collider::PlayersDollCollider, ui::UISystem, world::static_object::VisualWave};



pub struct VisualElement<'a> {
    pub transform: &'a Transform,
    pub static_objects: Option<&'a Vec<StaticObject>>,
    pub coloring_areas: Option<&'a Vec<ColoringArea>>,
    pub volume_areas: Option<&'a Vec<VolumeArea>>,
    pub waves: Option<&'a Vec<VisualWave>>,
    pub player: Option<(&'a PlayersDollCollider, Team)>,
    pub child_visual_elem: Option<&'a ChildVisualElement>,
}

pub struct ChildVisualElement {
    pub static_objects: Option<Vec<StaticObject>>,
    pub coloring_areas: Option<Vec<ColoringArea>>,
    pub volume_areas: Option<Vec<VolumeArea>>,
    pub waves: Option<Vec<VisualWave>>,
    pub player: Option<(PlayersDollCollider, Team)>,
}


struct RenderQualityData
{
    shadows_enabled: bool
}

pub struct RenderSystem {
    render_data: RenderData,
    pub window: Window,
    renderer: Arc<Mutex<Renderer>>,
    resize_buffers_signal: Arc<Mutex<bool>>,

    generated_raymarch_shader: bool,

    render_quality_data: RenderQualityData,
}



impl RenderSystem {
    pub async fn new(
        window: Window,
        world: &World,
        time: &TimeSystem,
        ui: &mut UISystem,
        #[cfg(not(target_arch="wasm32"))]
        runtime: &mut Runtime,
        it_is_2d_3d_example: bool,
        with_ui_renderer: bool,
        with_generated_raymarch_shader: bool,
        specific_backend: Option<Backend>,
    ) -> Self {
        
        let render_data = RenderData::new(world, time, &window);
        
        let renderer = Arc::new(
            Mutex::new(
                Renderer::new(
                    &window,
                    &render_data,
                    ui,
                    0.008,
                    // time.target_frame_duration.as_secs_f64(),
                    world.players_settings.screen_resolution_scale,
                    &world.level.visual_settings_of_environment.sky_box_name,
                    it_is_2d_3d_example,
                    with_ui_renderer,
                    with_generated_raymarch_shader,
                    specific_backend,
                ).await
            )
        );

        let outdated_signal_mutex = Arc::new(Mutex::new(false));

        let resize_buffers_signal = outdated_signal_mutex.clone();
        // spawn async tusk for renderer
        #[cfg(not(target_arch="wasm32"))]
        let async_renderer = renderer.clone();
        #[cfg(not(target_arch="wasm32"))]
        runtime.spawn(async move {

            #[cfg(target_os = "windows")]
            unsafe {windows_sys::Win32::Media::timeBeginPeriod(1);}

            loop {
                match async_renderer.try_lock() {
                    Ok(mut renderer_lock) => {
                        if let Err(err) = renderer_lock.render(/*&self.window*/) {
                            match err {
                                wgpu::SurfaceError::Lost => *resize_buffers_signal.lock().unwrap() = true,
                
                                wgpu::SurfaceError::Outdated => *resize_buffers_signal.lock().unwrap() = true,
                                
                                // The system is out of memory, we should probably quit
                                wgpu::SurfaceError::OutOfMemory => panic!("Out of GPU memory"),

                                _ => log::error!("{:?}", err),
                            }
                        }
                    }
                    Err(_) => {}
                }

                tokio::time::sleep(tokio::time::Duration::from_micros(4166)).await;
            }
        });


        log::info!("render system: renderer init");

        let render_quality_data = RenderQualityData {
            shadows_enabled: true,
        };
        
        RenderSystem {
            window,
            renderer,

            render_data,
            resize_buffers_signal: outdated_signal_mutex,

            generated_raymarch_shader: with_generated_raymarch_shader,

            render_quality_data,
        }
    }

    pub fn process_player_input(
        &mut self,
        frame_input: ActionsFrameState,
    )
    {
        if frame_input.increase_render_quality.is_action_just_pressed()
        {
            self.renderer
                .lock()
                .unwrap()
                .increase_raymarch_target_texture_scale_factor();
        }
        else if frame_input.decrease_render_quality.is_action_just_pressed()
        {
            self.renderer
                .lock()
                .unwrap()
                .decrease_raymarch_target_texture_scale_factor();
        }

        if frame_input.shadows_toggle.is_action_just_pressed()
        {
            self.render_quality_data.shadows_enabled = !self.render_quality_data.shadows_enabled;
        }
    }


    pub fn send_data_to_renderer(
        &mut self,
        world: &World,
        time: &TimeSystem,
        ui: &UISystem,
    ) {
        if *self.resize_buffers_signal.lock().unwrap() == true
        {
            *self.resize_buffers_signal.lock().unwrap() = false;

            self.resize_frame_buffer();
        }

        self.render_data.update_dynamic_render_data(
            world,
            time,
            &self.window,
            &self.render_data.static_data.static_bounding_box.clone(),
            self.generated_raymarch_shader,
            &self.render_quality_data,
        );

        let mut renderer_lock = self.renderer.lock().unwrap();

        ui.write_buffers_ui(
            &renderer_lock.queue,
            self.window.inner_size().width as f32 /
            self.window.inner_size().height as f32
        );

        renderer_lock.queue.write_buffer(
            &renderer_lock.other_dynamic_data_buffer,
            0,
            bytemuck::cast_slice(&[self.render_data.dynamic_data.other_dynamic_data]),
        );

        renderer_lock.queue.write_buffer(
            &renderer_lock.dynamic_negative_shapes_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.dynamic_shapes_data.negative.as_slice()),
        );

        renderer_lock.queue.write_buffer(
            &renderer_lock.dynamic_normal_shapes_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.dynamic_shapes_data.normal.as_slice()),
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

