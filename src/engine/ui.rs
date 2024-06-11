use std::collections::HashMap;

use wgpu::Buffer;

#[repr(C)]
pub struct ProgressBarUniform {
    pub vertex_scale: [f32;2],
    pub vertex_translation: [f32;2],
    pub empty_bytes: [f32;2],
    pub vertex_rotation: f32,
    pub bar_value: f32,
}

#[repr(C)]
pub struct ImageUniform {
    pub vertex_scale: [f32;2],
    pub vertex_translation: [f32;2],
    pub empty_bytes: [f32;3],
    pub vertex_rotation: f32,
}

pub struct ProgressBar {
    vertex_scale: [f32;2],
    vertex_translation: [f32;2],
    vertex_rotation: f32,

    bar_texture: String,
    bar_mask_texture: String,

    bar_value: f32,

    progress_bar_uniform_buffer: Buffer,
}

pub struct Image {
    vertex_scale: [f32;2],
    vertex_translation: [f32;2],
    vertex_rotation: f32,

    texture: String,

    image_uniform_buffer: Buffer,
} 

pub enum UIElement {
    Image(Image),
    ProgressBar(ProgressBar),
}

#[derive(PartialEq, Eq, Hash)]
pub enum ConcreteUIElement {
    HeathBar,
    EnergyGunEnergyBar,
    MachinegunEnergyBar,
    Crosshair,
    WRotationPointer,
    WHeightPointer,
}


pub struct UISystem {
    texture_sources: HashMap<String, &'static [u8]>,

    ui_elements: HashMap<ConcreteUIElement, UIElement>,
}

impl UISystem {

    pub fn get_ui_element(
        &mut self,
        element: ConcreteUIElement
    ) -> &mut UIElement {
        self.ui_elements.get_mut(&element)
            .expect("Some concrete UI element is not exist")
    }

    pub fn write_buffers_ui(&mut self) {

    }
}