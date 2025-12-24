// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2025  Timofei Molokov

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

pub mod render_data;
pub mod camera;
pub mod raymarch_shader_generator;
mod renderer;
mod ui_renderer;

use std::{sync::{Arc, Mutex}, time::Duration};

use crate::{
    engine::{
        render::renderer::RendererBuffers, time::TimeSystem, world::{
            World, static_object::{
                ColoringArea, StaticObject, VolumeArea
            }
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
use wgpu::{Backend, Device, PipelineLayout, Queue, RenderPipeline};
use winit::{dpi::PhysicalSize, window::Window};

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

#[derive(Clone)]
pub struct MainShaderRenderPipelineBuilderKit
{
    pub device: Device,
    pub vertex_desc: wgpu::VertexBufferLayout<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub raymarch_render_pipeline_layout: PipelineLayout,
}

pub struct RenderQualityData
{
    shadows_enabled: bool
}

pub struct RenderSystem {
    render_data: RenderData,
    pub window: Arc<Window>,
    renderer: Arc<Mutex<Renderer>>,
    renderer_queue: Arc<Queue>,
    buffers: RendererBuffers,
    main_raymarch_shader_pipeline: Arc<Mutex<Option<RenderPipeline>>>,
    render_quality_data: RenderQualityData,
    window_size: PhysicalSize<u32>,
    is_generated_raymarch_shader: bool,
    pub render_pipeline_builder_kit: MainShaderRenderPipelineBuilderKit,
}



impl RenderSystem
{
    pub async fn new(
        window: Window,
        time: &TimeSystem,
        ui: &mut UISystem,
        #[cfg(not(target_arch="wasm32"))]
        runtime: &mut Runtime,
        with_ui_renderer: bool,
        specific_backend: Option<Backend>,
    ) -> Self
    {    
        let render_data = RenderData::new(time, &window);

        let main_raymarch_shader_pipeline = Arc::new(Mutex::new(None));
        
        let (renderer, buffers, render_pipeline_builder_kit) = Renderer::new(
                &window,
                &render_data,
                ui,
                0.008,
                // time.target_frame_duration.as_secs_f64(),
                1.0,
                with_ui_renderer,
                specific_backend,
                main_raymarch_shader_pipeline.clone(),
            ).await;

        let renderer_queue = renderer.queue.clone();

        let renderer = Arc::new(Mutex::new(renderer));
        // spawn async tusk for renderer
        #[cfg(not(target_arch="wasm32"))]
        // let async_renderer = renderer.clone();
        #[cfg(not(target_arch="wasm32"))]
        let async_renderer = renderer.clone();

        let window = Arc::new(window);
        let window_for_renderer = window.clone();

        runtime.spawn(async move {

            // #[cfg(target_os = "windows")]
            // unsafe {windows_sys::Win32::Media::timeBeginPeriod(1);}

            loop
            {
                let timestamp = std::time::Instant::now();

                match async_renderer.try_lock()
                {
                    Ok(mut async_renderer) =>
                    {
                        if let Err(err) = async_renderer.render(window_for_renderer.clone()) {
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
            renderer_queue,
            buffers,
            render_data,
            main_raymarch_shader_pipeline,
            render_quality_data,
            window_size,
            is_generated_raymarch_shader: false,
            render_pipeline_builder_kit,
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
            self.is_generated_raymarch_shader,
            &self.render_quality_data,
        );

        ui.write_buffers_ui(
            self.renderer_queue.clone(),
            self.window_size.width as f32 /
            self.window_size.height as f32
        );
        

        self.renderer_queue.write_buffer(
            &self.buffers.other_dynamic_data_buffer,
            0,
            bytemuck::cast_slice(&[self.render_data.dynamic_data.other_dynamic_data]),
        );

        self.renderer_queue.write_buffer(
            &self.buffers.dynamic_negative_shapes_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.dynamic_shapes_data.negative.as_slice()),
        );

        self.renderer_queue.write_buffer(
            &self.buffers.dynamic_normal_shapes_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.dynamic_shapes_data.normal.as_slice()),
        );

        self.renderer_queue.write_buffer(
            &self.buffers.dynamic_stickiness_shapes_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.dynamic_shapes_data.stickiness.as_slice()),
        );

        self.renderer_queue.write_buffer(
            &self.buffers.dynamic_neg_stickiness_shapes_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.dynamic_shapes_data.neg_stickiness.as_slice()),
        );

        self.renderer_queue.write_buffer(
            &self.buffers.dynamic_undestroyable_shapes_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.dynamic_shapes_data.undestroyable.as_slice()),
        );

        self.renderer_queue.write_buffer(
            &self.buffers.dynamic_undestroyable_stickiness_shapes_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.dynamic_shapes_data.undestroyable_stickiness.as_slice()),
        );
        
        self.renderer_queue.write_buffer(
            &self.buffers.spherical_areas_data_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.spherical_areas_data.as_slice()),
        );

        self.renderer_queue.write_buffer(
            &self.buffers.beam_areas_data_buffer,
            0,
            bytemuck::cast_slice(self.render_data.dynamic_data.beam_areas_data.as_slice()),
        );

        self.renderer_queue.write_buffer(
            &self.buffers.player_forms_data_buffer,
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



    pub fn resize_frame_buffer(&mut self)
    {    
        self.window_size = self.window.inner_size();
        
        self.renderer
            .lock()
            .unwrap()
            .resize(self.window_size);
    }


    pub fn set_new_level(&mut self, world: &mut World)
    {
        self.is_generated_raymarch_shader = world.level.as_ref().unwrap().is_generated_raymarch_shader;

        self.render_data.static_data.update(world);

        self.renderer_queue.write_buffer(
            &self.buffers.other_static_data_buffer,
            0,
            bytemuck::cast_slice(&[self.render_data.static_data.other_static_data]),
        );

        *self.main_raymarch_shader_pipeline.lock().unwrap() = Some(
            world.level.as_mut().unwrap().main_shader.take().expect(
                "level have not main shader"
            )
        )
    }
}

