use std::{collections::{hash_map::IterMut, HashMap}, sync::{Arc, Mutex}};

use glam::Vec2;
use wgpu::{Buffer, Queue};

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
    pub transparency: f32,
    pub empty_bytes: f32,
    pub rotation_around_rect_center: f32,
    pub rotation_around_screen_center: f32,
}

pub enum UIElement {
    Image(UIImage),
    ProgressBar(UIProgressBar),
}

#[derive(PartialEq, Eq, Hash)]
pub enum UIElementType {
    HeathBar,
    EnergyGunEnergyBar,
    MachinegunEnergyBar,
    Crosshair,
    WRotationPointer,
    WHeightPointer,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum TextureType {
    HeathBar,
    HeathBarMask,
    EnergyGunEnergyBar,
    EnergyGunEnergyBarMask,
    MachinegunEnergyBar,
    MachinegunEnergyBarMask,
    Crosshair,
    WRotationPointer,
    WHeightPointer,
}



pub struct UISystem {
    pub texture_sources: HashMap<TextureType, &'static [u8]>,

    pub ui_elements: HashMap<UIElementType, UIElement>,
}

impl UISystem {

    pub fn new() -> UISystem {

        let mut texture_sources = HashMap::with_capacity(10);

        texture_sources.insert(
            TextureType::Crosshair,
            include_bytes!("../assets/textures/crosshair.png").as_slice()
        );
        texture_sources.insert(
            TextureType::HeathBar,
            include_bytes!("../assets/textures/healthbar.png").as_slice()
        );
        texture_sources.insert(
            TextureType::HeathBarMask,
            include_bytes!("../assets/textures/healthbar_mask.png").as_slice()
        );
        texture_sources.insert(
            TextureType::EnergyGunEnergyBarMask,
            include_bytes!("../assets/textures/energybar_mask.png").as_slice()
        );
        texture_sources.insert(
            TextureType::WRotationPointer,
            include_bytes!("../assets/textures/crosshair_w_rotation_pointer.png").as_slice()
        );
        texture_sources.insert(
            TextureType::WHeightPointer,
            include_bytes!("../assets/textures/crosshair_w_position_pointer.png").as_slice()
        );

        let mut ui_elements = HashMap::with_capacity(10);

        ui_elements.insert(
            UIElementType::Crosshair,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterCenter,
                            position: Vec2::ZERO,
                            size: RectSize::LockedHeight(
                                0.25
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                        },
                        true,
                    ),
                    TextureType::Crosshair
                )
            )
        );
        ui_elements.insert(
            UIElementType::WRotationPointer,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterCenter,
                            position: Vec2::ZERO,
                            size: RectSize::LockedHeight(
                                0.25
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                        },
                        true,
                    ),
                    TextureType::WRotationPointer
                )
            )
        );
        ui_elements.insert(
            UIElementType::WHeightPointer,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterCenter,
                            position: Vec2::ZERO,
                            size: RectSize::LockedHeight(
                                0.25
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                        },
                        true,
                    ),
                    TextureType::WHeightPointer
                )
            )
        );
        ui_elements.insert(
            UIElementType::HeathBar,
            UIElement::ProgressBar(
                UIProgressBar::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::DownLeft,
                            position: Vec2::new(-1.0, -1.0),
                            size: RectSize::LockedWight(
                                0.2
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                        },
                        true,
                    ),
                    TextureType::HeathBar,
                    TextureType::HeathBarMask,
                    0.02,
                    0.98,
                    ProgressBarDirection::LeftRight,
                )
            )
        );
        ui_elements.insert(
            UIElementType::EnergyGunEnergyBar,
            UIElement::ProgressBar(
                UIProgressBar::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::DownRight,
                            position: Vec2::new(1.0, -1.0),
                            size: RectSize::LockedWight(
                                0.2
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                        },
                        true,
                    ),
                    TextureType::HeathBar,
                    TextureType::EnergyGunEnergyBarMask,
                    0.98,
                    0.02,
                    ProgressBarDirection::RightLeft,
                )
            )
        );

        UISystem {
            ui_elements,
            texture_sources,
        }

    }


    pub fn get_texture_source(&self, texture_type: &TextureType) -> &[u8] {
        self.texture_sources
            .get(texture_type)
            .expect("ui system have not some texture's source")
    }


    pub fn get_ui_element(
        &mut self,
        element: UIElementType
    ) -> &mut UIElement {
        self.ui_elements
            .get_mut(&element)
            .expect("Some concrete UI element is not exist")
    }


    pub fn iter_mut_ui_elems(
        &mut self
    ) -> IterMut<UIElementType, UIElement> {
        self.ui_elements.iter_mut()
    }


    pub fn write_buffers_ui(&self, queue: &Queue, screen_aspect: f32) {

        for (_, ui_elem) in &self.ui_elements {
            match ui_elem {
                UIElement::Image(elem) => {
                    queue.write_buffer(
                        elem.rect_transform_buffer
                            .as_ref()
                            .expect("UI Image have not rect transform buffer"),
                        0,
                        bytemuck::cast_slice(&[
                            elem.ui_data.rect
                                .get_rect_transform_uniform(
                                    elem
                                        .texture_aspect
                                        .expect("UI Image have not texture aspect"),
                                    screen_aspect
                        )]),
                    );
                }
                UIElement::ProgressBar(elem) => {
                    queue.write_buffer(
                        &elem.rect_transform_buffer
                            .as_ref()
                            .expect("UI Progress bar have not rect transform buffer"),
                        0,
                        bytemuck::cast_slice(&[
                            elem.ui_data.rect
                                .get_rect_transform_uniform(
                                    elem
                                        .texture_aspect
                                        .expect("UI Progress bar have not texture aspect"),
                                    screen_aspect,
                        )]),
                    );
                    queue.write_buffer(
                        &elem.progress_bar_value_buffer
                            .as_ref()
                            .expect("UI Progress bar have not value buffer"),
                        0,
                        bytemuck::cast_slice(&[
                            elem.get_progress_bar_uniform()
                        ]),
                    );
                }
            }
        }
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

    pub transparency: f32
}

impl UIRect {
    pub fn get_rect_transform_uniform(
        &self,
        texture_aspect: f32,
        screen_aspect: f32,
    ) -> RectTransformUniform {

        let scale = {
            match self.size {
                RectSize::LockedBoth(x, y) => {
                    [x, y]
                },
                RectSize::LockedHeight(y) => {
                    [((y*texture_aspect)/screen_aspect), y]
                },
                RectSize::LockedWight(x) => {
                    [x, ((x/texture_aspect)*screen_aspect)]
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
                    [self.position.x - scale[0],
                    self.position.y - scale[1]]
                }
                RectAnchor::TopLeft => {
                    [self.position.x + scale[0],
                    self.position.y - scale[1]]
                }
                RectAnchor::CenterTop => {
                    [self.position.x,
                    self.position.y - scale[1]]
                }
                RectAnchor::DownLeft => {
                    [self.position.x + scale[0],
                    self.position.y + scale[1]]
                }
                RectAnchor::DownRight => {
                    [self.position.x - scale[0],
                    self.position.y + scale[1]]
                }
                RectAnchor::CenterDown => {
                    [self.position.x,
                    self.position.y + scale[1]]
                }
                RectAnchor::CenterLeft => {
                    [self.position.x + scale[0],
                    self.position.y]
                }
                RectAnchor::CenterRight => {
                    [self.position.x - scale[0],
                    self.position.y]
                }
            }
        };
        
        RectTransformUniform {
            rotation_around_rect_center: self.rotation_around_rect_center,
            rotation_around_screen_center: self.rotation_around_screen_center,
            transparency: self.transparency,
            empty_bytes: 0.0,
            scale,
            translation,
        }
    }


}

// pub struct Texture {
//     texture_type: TextureType,
//     source: &'static [u8],
//     di
// }

pub struct UIData {
    pub is_visible: Arc<Mutex<bool>>,
    pub rect: UIRect,
    pub need_to_redraw: bool
}

impl UIData {
    pub fn new(
        rect: UIRect,
        is_visible: bool,
    ) -> Self {
        let is_visible =  Arc::new(Mutex::new(is_visible));

        UIData {
            is_visible,
            rect,
            need_to_redraw: true
        }
    }

    pub fn set_is_visible(&mut self, is_visible: bool) {
        *self.is_visible.lock().unwrap() = is_visible;
    }

    pub fn get_is_visible_cloned_arc(&self) -> Arc<Mutex<bool>> {
        self.is_visible.clone()
    }

    pub fn set_transparecy(&mut self, transparency: f32) {
        self.rect.transparency = transparency;
    }

    pub fn get_transparecy(&self) -> f32 {
        self.rect.transparency
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.rect.position = position;
    }

    pub fn get_position(&self) -> Vec2 {
        self.rect.position
    }

    pub fn set_rotation_around_screen_center(&mut self, rotation_around_screen_center: f32) {
        self.rect.rotation_around_screen_center = rotation_around_screen_center;
    }

    pub fn get_rotation_around_screen_center(&self) -> f32 {
        self.rect.rotation_around_screen_center
    }

    pub fn set_rotation_around_rect_center(&mut self, rotation_around_rect_center: f32) {
        self.rect.rotation_around_rect_center = rotation_around_rect_center;
    }

    pub fn get_rotation_around_rect_center(&self) -> f32 {
        self.rect.rotation_around_rect_center
    }
}

pub struct UIImage {
    pub ui_data: UIData,
    texture: TextureType,

    rect_transform_buffer: Option<Buffer>,
    texture_aspect: Option<f32>,
    texture_size: Option<Vec2>,
}

impl UIImage {
    fn new(
        ui_data: UIData,
        texture: TextureType,
    ) -> Self {
        
        UIImage {
            ui_data,
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
        self.ui_data.set_is_visible(is_visible);
    }

    pub fn get_is_visible_cloned_arc(&self) -> Arc<Mutex<bool>> {
        self.ui_data.get_is_visible_cloned_arc()
    }

    pub fn set_transparecy(&mut self, transparency: f32) {
        self.ui_data.set_transparecy(transparency);
    }

    pub fn get_transparecy(&self) -> f32 {
        self.ui_data.get_transparecy()
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.ui_data.set_position(position);
    }

    pub fn get_position(&self) -> Vec2 {
        self.ui_data.get_position()
    }

    pub fn set_rotation_around_screen_center(&mut self, rotation_around_screen_center: f32) {
        self.ui_data.set_rotation_around_screen_center(rotation_around_screen_center);
    }

    pub fn get_rotation_around_screen_center(&self) -> f32 {
        self.ui_data.get_rotation_around_screen_center()
    }

    pub fn set_rotation_around_rect_center(&mut self, rotation_around_rect_center: f32) {
        self.ui_data.set_rotation_around_rect_center(rotation_around_rect_center);
    }

    pub fn get_rotation_around_rect_center(&self) -> f32 {
        self.ui_data.get_rotation_around_rect_center()
    }
}

pub enum ProgressBarDirection {
    RightLeft,
    LeftRight,
    DownTop,
    TopDown,
}


pub struct UIProgressBar {
    pub ui_data: UIData,
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
        ui_data: UIData,
        texture: TextureType,
        bar_mask: TextureType,

        from: f32,
        to: f32,
        direction: ProgressBarDirection,
    ) -> Self {

        UIProgressBar {
            ui_data,
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

    pub fn set_bar_value(&mut self, value: f32) {
        self.bar_value = value;
    }

    pub fn get_bar_value(&self) -> f32 {
        self.bar_value
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
        self.ui_data.set_is_visible(is_visible);
    }

    pub fn get_is_visible_cloned_arc(&self) -> Arc<Mutex<bool>> {
        self.ui_data.get_is_visible_cloned_arc()
    }
}
