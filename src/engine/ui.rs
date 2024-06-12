use std::{collections::{hash_map::IterMut, HashMap}, sync::{Arc, Mutex}};

use fyrox_core::math::Rect;
use glam::Vec2;
use wgpu::Buffer;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ProgressBarUniform {
    pub value: f32,
    pub from: f32,
    pub to: f32,
    pub direction: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectTransformUniform {
    pub scale: [f32;2],
    pub translation: [f32;2],
    pub empty_bytes: [f32;2],
    pub rotation_around_rect_center: f32,
    pub rotation_around_screen_center: f32,
}

pub enum UIElement {
    Image(UIImage),
    ProgressBar(UIProgressBar),
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

#[derive(PartialEq, Eq, Hash)]
pub enum TextureType {
    HeathBar,
    EnergyGunEnergyBar,
    EnergyGunEnergyBarMask,
    MachinegunEnergyBar,
    MachinegunEnergyBarMask,
    Crosshair,
    WRotationPointer,
    WHeightPointer,
}



pub struct UISystem {
    texture_sources: HashMap<TextureType, &'static [u8]>,

    pub ui_elements: HashMap<ConcreteUIElement, UIElement>,
}

impl UISystem {

    pub fn new() -> UISystem {

        let mut texture_sources = HashMap::with_capacity(10);

        texture_sources.insert(
            TextureType::Crosshair,
            include_bytes!("../assets/textures/crosshair.png").as_slice()
        );

        let mut ui_elements = HashMap::with_capacity(10);

        ui_elements.insert(
            ConcreteUIElement::Crosshair,
            UIElement::Image(
                UIImage::new(
                    UIRect {
                        anchor: RectAnchor::CenterCenter,
                        position: Vec2::ZERO,
                        size: RectSize::LockedHeight(
                            0.3
                        ),
                        rotation_around_rect_center: 0.0,
                        rotation_around_screen_center: 0.0,

                        is_visible: Arc::new(Mutex::new(true)),
                    },
                    TextureType::Crosshair
                )
            )
        );

        UISystem {
            ui_elements,
            texture_sources,
        }

    }

    pub fn get_texture_source(&self, texture_type: &TextureType) -> &[u8] {
        self.texture_sources.get(texture_type).expect("ui system have not some texture's source")
    }

    pub fn get_ui_element(
        &mut self,
        element: ConcreteUIElement
    ) -> &mut UIElement {
        self.ui_elements.get_mut(&element)
            .expect("Some concrete UI element is not exist")
    }

    pub fn iter_mut_ui_elems(
        &mut self
    ) -> IterMut<ConcreteUIElement, UIElement> {
        self.ui_elements.iter_mut()
    }

    pub fn write_buffers_ui(&mut self) {

    }
}

// origin of rect in rect space
pub enum RectAnchor {
    TopRight,
    TopLeft,
    DownRight,
    DownLeft,
    CenterLeft,
    CenterRight,
    CenterTop,
    CenterDown,
    CenterCenter,
}

pub enum RectSize {
    // it's meant rect's height will be static
    // and width will be calculated from texture's aspect ratio
    LockedHeight(f32),
    // the same but width is static
    LockedWight(f32),
    // size of rect in screen space will be static
    LockedBoth(f32,f32),
}

pub struct UIRect {
    // origin of rect in rect space
    pub anchor: RectAnchor,
    // position of rect in screen space
    pub position: Vec2,
    // size of rect in screen space
    pub size: RectSize,
    pub rotation_around_rect_center: f32,
    pub rotation_around_screen_center: f32,

    pub is_visible: Arc<Mutex<bool>>,
}

impl UIRect {
    pub fn get_rect_transform_uniform(
        &self,
        aspect: f32
    ) -> RectTransformUniform {

        let scale = {
            match self.size {
                RectSize::LockedBoth(x, y) => {
                    [x*2.0, y*2.0]
                },
                RectSize::LockedHeight(y) => {
                    [y*aspect*2.0, y*2.0]
                },
                RectSize::LockedWight(x) => {
                    [x*2.0, (x/aspect)*2.0]
                }
            }
        };

        //         ________________
        //         |              + (1,1)
        //         | wgpu screeen |
        //         |              |
        //         |      +(0,0)  |
        //         |              |
        //         |              |
        // (-1,-1) +______________|

        let translation = {
            match self.anchor {
                RectAnchor::CenterCenter => {
                    self.position.to_array()
                }
                RectAnchor::TopRight => {
                    [self.position.x - scale[0]*0.5,
                    self.position.y - scale[1]*0.5]
                }
                RectAnchor::TopLeft => {
                    [self.position.x + scale[0]*0.5,
                    self.position.y - scale[1]*0.5]
                }
                RectAnchor::CenterTop => {
                    [self.position.x,
                    self.position.y - scale[1]*0.5]
                }
                RectAnchor::DownLeft => {
                    [self.position.x + scale[0]*0.5,
                    self.position.y + scale[1]*0.5]
                }
                RectAnchor::DownRight => {
                    [self.position.x - scale[0]*0.5,
                    self.position.y + scale[1]*0.5]
                }
                RectAnchor::CenterDown => {
                    [self.position.x,
                    self.position.y + scale[1]*0.5]
                }
                RectAnchor::CenterLeft => {
                    [self.position.x + scale[0]*0.5,
                    self.position.y]
                }
                RectAnchor::CenterRight => {
                    [self.position.x - scale[0]*0.5,
                    self.position.y]
                }
            }
        };
        
        RectTransformUniform {
            rotation_around_rect_center: self.rotation_around_rect_center,
            rotation_around_screen_center: self.rotation_around_screen_center,
            empty_bytes: [0.0,0.0],
            scale,
            translation,
        }
    }

    pub fn set_is_visible(&mut self, is_visible: bool) {
        *self.is_visible.lock().unwrap() = is_visible;
    }

    pub fn get_is_visible_cloned_arc(&self) -> Arc<Mutex<bool>> {
        self.is_visible.clone()
    }
}

pub struct UIImage {
    pub rect: UIRect,
    texture: TextureType,

    rect_transform_buffer: Option<Buffer>,
    texture_aspect: Option<f32>,
    texture_size: Option<Vec2>,
}

impl UIImage {
    fn new(
        rect: UIRect,
        texture: TextureType,
    ) -> Self {
        
        UIImage {
            rect,
            texture,

            rect_transform_buffer: None,
            texture_aspect: None,
            texture_size: None,
        }
    }

    pub fn get_texture_type(&self) -> &TextureType {
        &self.texture
    }

    pub fn initialize(
        &mut self,
        texture_size: Vec2,
        texture_aspect: f32,
        rect_transform_buffer: Buffer,
    ) {
        self.rect_transform_buffer = Some(rect_transform_buffer);
        self.texture_aspect = Some(texture_aspect);
        self.texture_size = Some(texture_size);
    }

    pub fn set_is_visible(&mut self, is_visible: bool) {
        self.rect.set_is_visible(is_visible);
    }

    pub fn get_is_visible_cloned_arc(&self) -> Arc<Mutex<bool>> {
        self.rect.get_is_visible_cloned_arc()
    }
}

pub enum ProgressBarDirection {
    RightLeft,
    LeftRight,
    DownTop,
    TopDown,
}


pub struct UIProgressBar {
    pub rect: UIRect,
    texture: TextureType,
    bar_mask: TextureType,

    bar_value: f32,
    from: f32,
    to: f32,
    direction: ProgressBarDirection,

    texture_size: Option<Vec2>,
    texture_aspect: Option<f32>,
    mask_texture_size: Option<Vec2>,
    mask_texture_aspect: Option<f32>,
    rect_transform_buffer: Option<Buffer>,
    progress_bar_value_buffer: Option<Buffer>,
}

impl UIProgressBar {
    fn new(
        rect: UIRect,
        texture: TextureType,
        bar_mask: TextureType,

        from: f32,
        to: f32,
        direction: ProgressBarDirection,
    ) -> Self {

        UIProgressBar {
            rect,
            texture,
            bar_mask,

            bar_value: 1.0,
            from,
            to,
            direction,

            texture_size: None,
            texture_aspect: None,
            mask_texture_size: None,
            mask_texture_aspect: None,
            rect_transform_buffer: None,
            progress_bar_value_buffer: None,
        }
    }

    pub fn get_texture_type(&self) -> &TextureType {
        &self.texture
    }

    pub fn get_mask_texture_type(&self) -> &TextureType {
        &self.bar_mask
    }

    pub fn initialize(
        &mut self,
        texture_size: Vec2,
        texture_aspect: f32,
        mask_texture_size: Vec2,
        mask_texture_aspect: f32,
        rect_transform_buffer: Buffer,
        progress_bar_value_buffer: Buffer,
    ) {
        self.texture_size = Some(texture_size);
        self.texture_aspect = Some(texture_aspect);
        self.mask_texture_size = Some(mask_texture_size);
        self.mask_texture_aspect = Some(mask_texture_aspect);
        self.rect_transform_buffer = Some(rect_transform_buffer);
        self.progress_bar_value_buffer = Some(progress_bar_value_buffer);
    }

    pub fn get_progress_bar_uniform(&self) -> ProgressBarUniform {

        let direction: f32 = {
            match self.direction {
                ProgressBarDirection::LeftRight => {0.0}
                ProgressBarDirection::RightLeft => {1.0}
                ProgressBarDirection::DownTop => {2.0}
                ProgressBarDirection::TopDown => {3.0}
            }
        };

        ProgressBarUniform {
            value: self.bar_value,
            from: self.from,
            to: self.to,
            direction,
        }
    }

    pub fn set_is_visible(&mut self, is_visible: bool) {
        self.rect.set_is_visible(is_visible);
    }

    pub fn get_is_visible_cloned_arc(&self) -> Arc<Mutex<bool>> {
        self.rect.get_is_visible_cloned_arc()
    }
}
