pub mod render_data;
pub mod camera;
pub mod raymarch_shader_generator;
pub mod ui_renderer;
mod renderer;

use std::sync::{Arc, Mutex};

use crate::{
    engine::{
        render::{render_data::DataForUniformBuffers, renderer::RendererBuffers}, time::TimeSystem, world::{
            static_object::{
                ColoringArea, StaticObject, VolumeArea
            },
            World
        }
    }, transform::Transform
};

use self::{
    renderer::Renderer,
    render_data::RenderData,
};

use client_server_protocol::Team;
#[cfg(not(target_arch="wasm32"))]
use tokio::runtime::Runtime;
use wgpu::{Backend, Buffer, Queue};
use winit::{dpi::PhysicalSize, monitor::{MonitorHandle, VideoModeHandle}, window::Window};

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


pub struct RenderQualityData
{
    shadows_enabled: bool
}



pub struct RenderSystem {
    render_data: RenderData,
    data_for_uniform_buffers: Arc<Mutex<DataForUniformBuffers>>,
    pub window: Arc<Window>,
    renderer: Arc<Mutex<Renderer>>,

    generated_raymarch_shader: bool,

    render_quality_data: RenderQualityData,

    window_size: PhysicalSize<u32>,
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
        
        let renderer = Renderer::new(
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
        ).await;

        let data_for_uniform_buffers = Arc::new(Mutex::new(DataForUniformBuffers::default()));
        let data_for_uniform_buffers_for_renderer = data_for_uniform_buffers.clone();

        let renderer = Arc::new(Mutex::new(renderer));
        // spawn async tusk for renderer
        #[cfg(not(target_arch="wasm32"))]
        // let async_renderer = renderer.clone();
        #[cfg(not(target_arch="wasm32"))]
        let async_renderer = renderer.clone();

        let window = Arc::new(window);
        let window_for_renderer = window.clone();

        runtime.spawn(async move {

            #[cfg(target_os = "windows")]
            unsafe {windows_sys::Win32::Media::timeBeginPeriod(1);}

            loop
            {
                let timestamp = std::time::Instant::now();

                match async_renderer.try_lock()
                {
                    Ok(mut async_renderer) =>
                    {
                        if let Err(err) = async_renderer.render(
                            window_for_renderer.clone(),
                            data_for_uniform_buffers_for_renderer.clone()
                        )
                        {
                            match err {
                                wgpu::SurfaceError::Lost => 
                                {
                                    async_renderer.resize(window_for_renderer.inner_size());
                                },
                
                                wgpu::SurfaceError::Outdated =>
                                {
                                    async_renderer.resize(window_for_renderer.inner_size());
                                },

                                wgpu::SurfaceError::Other =>
                                {
                                    async_renderer.resize(window_for_renderer.inner_size());
                                }
                                
                                // The system is out of memory, we should probably quit
                                wgpu::SurfaceError::OutOfMemory => panic!("Out of GPU memory"),

                                _ => log::error!("{:?}", err),
                            }
                        }
                    }
                    Err(_) =>
                    {

                    }
                }

                if timestamp.elapsed().as_millis() < 3
                {
                    tokio::time::sleep(tokio::time::Duration::from_micros(3122)).await;
                }
                
                // println!("render loop time {}", timestamp.elapsed().as_secs_f32());
            }
        });


        log::info!("render system: renderer init");

        let render_quality_data = RenderQualityData {
            shadows_enabled: true,
        };
        
        let window_size = window.inner_size();

        RenderSystem {
            window,
            renderer,

            render_data,
            data_for_uniform_buffers,

            generated_raymarch_shader: with_generated_raymarch_shader,

            render_quality_data,
            window_size,
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
    )
    {


        self.render_data.update_dynamic_render_data(
            world,
            time,
            self.window_size,
            &self.render_data.static_data.static_bounding_box.clone(),
            self.generated_raymarch_shader,
            &self.render_quality_data,
        );

        ui.write_buffers_ui(
            self.window_size.width as f32 / self.window_size.height as f32
        );

        let mut data = self.data_for_uniform_buffers.lock().unwrap();

        data.beam_areas_data = self.render_data.dynamic_data.beam_areas_data.clone();
        data.dynamic_shapes_data = self.render_data.dynamic_data.dynamic_shapes_data.clone();
        data.other_dynamic_data = self.render_data.dynamic_data.other_dynamic_data.clone();
        data.player_forms_data = self.render_data.dynamic_data.player_forms_data.clone();
        data.spherical_areas_data = self.render_data.dynamic_data.spherical_areas_data.clone();

        drop(data);



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
        self.window_size = self.window.inner_size();

        self.renderer
            .lock()
            .unwrap()
            .resize(self.window_size);
    }
}

