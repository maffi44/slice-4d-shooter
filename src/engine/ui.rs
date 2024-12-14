use std::{
    collections::{
        hash_map::IterMut,
        HashMap
    },
    sync::{
        Arc,
        Mutex
    }
};

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

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScannerDataUniform {
    empty_byte0: u32,
    empty_byte1: u32,
    empty_byte2: u32,
    orientation: u32,
}

pub enum UIElement {
    Image(UIImage),
    ProgressBar(UIProgressBar),
    ScannerDisplay(UIScannerDisplay),
}

impl UIElement
{
    pub fn get_ui_data_mut(&mut self) -> &mut UIData
    {
        match self
        {
            UIElement::Image(uiimage) =>
            {
                &mut uiimage.ui_data
            }

            UIElement::ProgressBar(uiprogress_bar) =>
            {
                &mut uiprogress_bar.ui_data
            }

            UIElement::ScannerDisplay(uiscanner_display) =>
            {
                &mut uiscanner_display.ui_data
            }
        }
    }

    pub fn get_ui_data(&self) -> &UIData
    {
        match self
        {
            UIElement::Image(uiimage) =>
            {
                &uiimage.ui_data
            }

            UIElement::ProgressBar(uiprogress_bar) =>
            {
                &uiprogress_bar.ui_data
            }

            UIElement::ScannerDisplay(uiscanner_display) =>
            {
                &uiscanner_display.ui_data
            }
        }   
    }
}

#[derive(PartialEq, Eq, Hash)]
pub enum UIElementType {
    HeathBarBlue,
    HeathBarRed,
    EnergyGunBarBlue,
    EnergyGunBarRed,
    EnergyGunImage,
    MachinegunBarBlue,
    MachinegunBarRed,
    MachinegunImage,
    Crosshair,
    CrosshairHitMark,
    ScannerBlue,
    ScannerRed,
    ScannerHPointerBlue,
    ScannerHPointerRed,
    ZXScannerArrowBlue,
    ZXScannerArrowRed,
    ZWScannerArrowBlue,
    ZWScannerArrowRed,
    LeftScannerDsiplayRed,
    LeftScannerDsiplayBlue,
    RightScannerDsiplayRed,
    RightScannerDsiplayBlue,
    RedFlagMark,
    BlueFlagMark,
    ScoreBar,
    FirstScoreMarkBlue,
    SecondScoreMarkBlue,
    ThirdScoreMarkBlue,
    FinalScoreMarkBlue,
    FirstScoreMarkRed,
    SecondScoreMarkRed,
    ThirdScoreMarkRed,
    FinalScoreMarkRed,
    MoveWBonusMarkFirst,
    MoveWBonusMarkSecond,
    RedTeamWinTitle,
    BlueTeamWinTitle,
    JoinRedTeamTitle,
    JoinBlueTeamTitle,
    BlueTeamBacklight,
    RedTeamBacklight,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum TextureType {
    HeathBarTexture,
    HeathBarMaskBlue,
    HeathBarMaskRed,
    EnergyGunBarTexture,
    EnergyGunImgTexture,
    MachinegunBarTexture,
    MachinegunImgTexture,
    GunBarMaskBlue,
    GunBarMaskRed,
    Crosshair,
    CrosshairHitMark,
    ScannerTextureBlue,
    ScannerTextureRed,
    ScannerPointerBlue,
    ScannerPointerRed,
    ScannerArrowBlue,
    ScannerArrowRed,
    BacklightBlue,
    BacklightRed,
    BlueTeamWinTitle,
    RedTeamWinTitle,
    JoinBlueTeamTitle,
    JoinRedTeamTitle,
    ScoreBar,
    BlueFlagMark,
    RedFlagMark,
    BlueScoreMark,
    RedScoreMark,
    BlueFinalMark,
    RedFinalMark,
    MoveWBonusImage,
}



pub struct UISystem {
    pub texture_sources: HashMap<TextureType, &'static [u8]>,

    pub ui_elements: HashMap<UIElementType, UIElement>,
}

impl UISystem {

    pub fn new() -> UISystem {

        let mut texture_sources = HashMap::with_capacity(10);

        texture_sources.insert(
            TextureType::HeathBarTexture,
            include_bytes!("../assets/textures/health_bar_texture_hud.png").as_slice()
        );
        texture_sources.insert(
            TextureType::HeathBarMaskBlue,
            include_bytes!("../assets/textures/blue_health_bar_mask_hud.png").as_slice()
        );
        texture_sources.insert(
            TextureType::HeathBarMaskRed,
            include_bytes!("../assets/textures/orange_health_bar_mask_hud.png").as_slice()
        );
        texture_sources.insert(
            TextureType::EnergyGunBarTexture,
            include_bytes!("../assets/textures/energy_gun_bar_texture_hud.png").as_slice()
        );
        texture_sources.insert(
            TextureType::EnergyGunImgTexture,
            include_bytes!("../assets/textures/energy_gun_image.png").as_slice()
        );
        texture_sources.insert(
            TextureType::MachinegunBarTexture,
            include_bytes!("../assets/textures/machinegun_bar_texture_hud.png").as_slice()
        );
        texture_sources.insert(
            TextureType::MachinegunImgTexture,
            include_bytes!("../assets/textures/machinegun_image.png").as_slice()
        );
        texture_sources.insert(
            TextureType::GunBarMaskBlue,
            include_bytes!("../assets/textures/blue_gun_bar_mask_hud.png").as_slice()
        );
        texture_sources.insert(
            TextureType::GunBarMaskRed,
            include_bytes!("../assets/textures/orange_gun_bar_mask_hud.png").as_slice()
        );
        texture_sources.insert(
            TextureType::Crosshair,
            include_bytes!("../assets/textures/crosshair_hud.png").as_slice()
        );
        texture_sources.insert(
            TextureType::CrosshairHitMark,
            include_bytes!("../assets/textures/crosshair_hit_mark.png").as_slice()
        );
        texture_sources.insert(
            TextureType::ScannerTextureBlue,
            include_bytes!("../assets/textures/blue_scanner_hud.png").as_slice()
        );
        texture_sources.insert(
            TextureType::ScannerTextureRed,
            include_bytes!("../assets/textures/orange_scanner_hud.png").as_slice()
        );
        texture_sources.insert(
            TextureType::ScannerPointerBlue,
            include_bytes!("../assets/textures/blue_scanner_pointer_hud.png").as_slice()
        );
        texture_sources.insert(
            TextureType::ScannerPointerRed,
            include_bytes!("../assets/textures/orange_scanner_pointer_hud.png").as_slice()
        );
        texture_sources.insert(
            TextureType::ScannerArrowBlue,
            include_bytes!("../assets/textures/blue_scanner_arrow_hud.png").as_slice()
        );
        texture_sources.insert(
            TextureType::ScannerArrowRed,
            include_bytes!("../assets/textures/orange_scanner_arrow_hud.png").as_slice()
        );
        texture_sources.insert(
            TextureType::BacklightBlue,
            include_bytes!("../assets/textures/blue_team_backlight.png").as_slice()
        );
        texture_sources.insert(
            TextureType::BacklightRed,
            include_bytes!("../assets/textures/orange_team_backlight.png").as_slice()
        );
        texture_sources.insert(
            TextureType::BlueTeamWinTitle,
            include_bytes!("../assets/textures/blue_team_win.png").as_slice()
        );
        texture_sources.insert(
            TextureType::RedTeamWinTitle,
            include_bytes!("../assets/textures/orange_team_win.png").as_slice()
        );
        texture_sources.insert(
            TextureType::JoinBlueTeamTitle,
            include_bytes!("../assets/textures/join_blue_team.png").as_slice()
        );
        texture_sources.insert(
            TextureType::JoinRedTeamTitle,
            include_bytes!("../assets/textures/join_orange_team.png").as_slice()
        );
        texture_sources.insert(
            TextureType::ScoreBar,
            include_bytes!("../assets/textures/score_bar.png").as_slice()
        );
        texture_sources.insert(
            TextureType::BlueFlagMark,
            include_bytes!("../assets/textures/blue_flag_mark.png").as_slice()
        );
        texture_sources.insert(
            TextureType::RedFlagMark,
            include_bytes!("../assets/textures/orange_flag_mark.png").as_slice()
        );
        texture_sources.insert(
            TextureType::BlueScoreMark,
            include_bytes!("../assets/textures/blue_score_mark.png").as_slice()
        );
        texture_sources.insert(
            TextureType::RedScoreMark,
            include_bytes!("../assets/textures/orange_score_mark.png").as_slice()
        );
        texture_sources.insert(
            TextureType::BlueFinalMark,
            include_bytes!("../assets/textures/blue_last_score_mark.png").as_slice()
        );
        texture_sources.insert(
            TextureType::RedFinalMark,
            include_bytes!("../assets/textures/orange_last_score_mark.png").as_slice()
        );
        texture_sources.insert(
            TextureType::MoveWBonusImage,
            include_bytes!("../assets/textures/move_w_bonus_img.png").as_slice()
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
                                0.04
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::Crosshair
                )
            )
        );
        ui_elements.insert(
            UIElementType::CrosshairHitMark,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterCenter,
                            position: Vec2::ZERO,
                            size: RectSize::LockedHeight(
                                0.025
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::CrosshairHitMark
                )
            )
        );
        ui_elements.insert(
            UIElementType::ScannerRed,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterDown,
                            position: Vec2::new(0.0, -1.0),
                            size: RectSize::LockedWight(
                                0.322
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::ScannerTextureRed
                )
            )
        );
        ui_elements.insert(
            UIElementType::ScannerBlue,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterDown,
                            position: Vec2::new(0.0, -1.0),
                            size: RectSize::LockedWight(
                                0.322
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::ScannerTextureBlue
                )
            )
        );
        ui_elements.insert(
            UIElementType::ScannerHPointerRed,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterCenter,
                            position: Vec2::new(0.0, -0.3),
                            size: RectSize::LockedBoth(
                                0.0315,
                                0.053
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 1,
                            transform_buffer: None,
                        },
                        false,
                        Some(UIElementType::ScannerRed),
                    ),
                    TextureType::ScannerPointerRed
                )
            )
        );
        ui_elements.insert(
            UIElementType::ScannerHPointerBlue,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterCenter,
                            position: Vec2::new(0.0, -0.3),
                            size: RectSize::LockedBoth(
                                0.0315,
                                0.053
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 1,
                            transform_buffer: None,
                        },
                        false,
                        Some(UIElementType::ScannerBlue),
                    ),
                    TextureType::ScannerPointerBlue
                )
            )
        );
        ui_elements.insert(
            UIElementType::ZXScannerArrowRed,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterCenter,
                            position: Vec2::new(-0.305, 0.063),
                            size: RectSize::LockedBoth(
                                0.229,
                                0.81
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 1,
                            transform_buffer: None,
                        },
                        false,
                        Some(UIElementType::ScannerRed),
                    ),
                    TextureType::ScannerArrowRed
                )
            )
        );
        ui_elements.insert(
            UIElementType::ZXScannerArrowBlue,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterCenter,
                            position: Vec2::new(-0.305, 0.063),
                            size: RectSize::LockedBoth(
                                0.229,
                                0.81
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 1,
                            transform_buffer: None,
                        },
                        false,
                        Some(UIElementType::ScannerBlue),
                    ),
                    TextureType::ScannerArrowBlue
                )
            )
        );
        ui_elements.insert(
            UIElementType::ZWScannerArrowRed,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterCenter,
                            position: Vec2::new(0.305, 0.063),
                            size: RectSize::LockedBoth(
                                0.229,
                                0.81
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 1,
                            transform_buffer: None,
                        },
                        false,
                        Some(UIElementType::ScannerRed),
                    ),
                    TextureType::ScannerArrowRed
                )
            )
        );
        ui_elements.insert(
            UIElementType::ZWScannerArrowBlue,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterCenter,
                            position: Vec2::new(0.305, 0.063),
                            size: RectSize::LockedBoth(
                                0.229,
                                0.81
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 1,
                            transform_buffer: None,
                        },
                        false,
                        Some(UIElementType::ScannerBlue),
                    ),
                    TextureType::ScannerArrowBlue
                )
            )
        );
        ui_elements.insert(
            UIElementType::LeftScannerDsiplayRed,
            UIElement::ScannerDisplay(
                UIScannerDisplay::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterCenter,
                            position: Vec2::new(-0.305, 0.063),
                            size: RectSize::LockedBoth(
                                0.229,
                                0.81
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 2,
                            transform_buffer: None,
                        },
                        false,
                        Some(UIElementType::ScannerRed),
                    ),
                    ScannerDisplayPlaneOrientation::ZX
                )
            )
        );
        ui_elements.insert(
            UIElementType::LeftScannerDsiplayBlue,
            UIElement::ScannerDisplay(
                UIScannerDisplay::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterCenter,
                            position: Vec2::new(-0.305, 0.063),
                            size: RectSize::LockedBoth(
                                0.229,
                                0.81
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 2,
                            transform_buffer: None,
                        },
                        false,
                        Some(UIElementType::ScannerBlue),
                    ),
                    ScannerDisplayPlaneOrientation::ZX
                )
            )
        );
        ui_elements.insert(
            UIElementType::RightScannerDsiplayRed,
            UIElement::ScannerDisplay(
                UIScannerDisplay::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterCenter,
                            position: Vec2::new(0.305, 0.063),
                            size: RectSize::LockedBoth(
                                0.229,
                                0.81
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 2,
                            transform_buffer: None,
                        },
                        false,
                        Some(UIElementType::ScannerRed),
                    ),
                    ScannerDisplayPlaneOrientation::ZW
                )
            )
        );
        ui_elements.insert(
            UIElementType::RightScannerDsiplayBlue,
            UIElement::ScannerDisplay(
                UIScannerDisplay::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterCenter,
                            position: Vec2::new(0.305, 0.063),
                            size: RectSize::LockedBoth(
                                0.229,
                                0.81
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 2,
                            transform_buffer: None,
                        },
                        false,
                        Some(UIElementType::ScannerBlue),
                    ),
                    ScannerDisplayPlaneOrientation::ZW
                )
            )
        );
        ui_elements.insert(
            UIElementType::HeathBarRed,
            UIElement::ProgressBar(
                UIProgressBar::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::DownLeft,
                            position: Vec2::new(-1.0, -1.0),
                            size: RectSize::LockedWight(
                                0.224
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::HeathBarTexture,
                    TextureType::HeathBarMaskRed,
                    0.17,
                    0.95,
                    ProgressBarDirection::LeftRight,
                )
            )
        );
        ui_elements.insert(
            UIElementType::HeathBarBlue,
            UIElement::ProgressBar(
                UIProgressBar::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::DownLeft,
                            position: Vec2::new(-1.0, -1.0),
                            size: RectSize::LockedWight(
                                0.224
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::HeathBarTexture,
                    TextureType::HeathBarMaskBlue,
                    0.17,
                    0.95,
                    ProgressBarDirection::LeftRight,
                )
            )
        );
        ui_elements.insert(
            UIElementType::EnergyGunBarRed,
            UIElement::ProgressBar(
                UIProgressBar::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::DownRight,
                            position: Vec2::new(1.0, -1.0),
                            size: RectSize::LockedWight(
                                0.224
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                        
                    ),
                    TextureType::EnergyGunBarTexture,
                    TextureType::GunBarMaskRed,
                    0.95,
                    0.17,
                    ProgressBarDirection::RightLeft,
                )
            )
        );
        ui_elements.insert(
            UIElementType::EnergyGunBarBlue,
            UIElement::ProgressBar(
                UIProgressBar::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::DownRight,
                            position: Vec2::new(1.0, -1.0),
                            size: RectSize::LockedWight(
                                0.224
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                        
                    ),
                    TextureType::EnergyGunBarTexture,
                    TextureType::GunBarMaskBlue,
                    0.95,
                    0.17,
                    ProgressBarDirection::RightLeft,
                )
            )
        );
        ui_elements.insert(
            UIElementType::EnergyGunImage,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::DownRight,
                            position: Vec2::new(1.0, -1.0),
                            size: RectSize::LockedWight(
                                0.224
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::EnergyGunImgTexture
                )
            )
        );
        ui_elements.insert(
            UIElementType::MachinegunBarRed,
            UIElement::ProgressBar(
                UIProgressBar::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::DownRight,
                            position: Vec2::new(1.0, -1.0),
                            size: RectSize::LockedWight(
                                0.224
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                        
                    ),
                    TextureType::MachinegunBarTexture,
                    TextureType::GunBarMaskRed,
                    0.95,
                    0.17,
                    ProgressBarDirection::RightLeft,
                )
            )
        );
        ui_elements.insert(
            UIElementType::MachinegunBarBlue,
            UIElement::ProgressBar(
                UIProgressBar::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::DownRight,
                            position: Vec2::new(1.0, -1.0),
                            size: RectSize::LockedWight(
                                0.224
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                        
                    ),
                    TextureType::MachinegunBarTexture,
                    TextureType::GunBarMaskBlue,
                    0.95,
                    0.17,
                    ProgressBarDirection::RightLeft,
                )
            )
        );
        ui_elements.insert(
            UIElementType::MachinegunImage,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::DownRight,
                            position: Vec2::new(1.0, -1.0),
                            size: RectSize::LockedWight(
                                0.224
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::MachinegunImgTexture
                )
            )
        );
        ui_elements.insert(
            UIElementType::MoveWBonusMarkFirst,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::DownLeft,
                            position: Vec2::new(-0.965, -1.0),
                            size: RectSize::LockedWight(
                                0.035
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::MoveWBonusImage
                )
            )
        );
        ui_elements.insert(
            UIElementType::MoveWBonusMarkSecond,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::DownLeft,
                            position: Vec2::new(-0.85, -1.0),
                            size: RectSize::LockedWight(
                                0.035
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::MoveWBonusImage
                )
            )
        );
        ui_elements.insert(
            UIElementType::ScoreBar,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterTop,
                            position: Vec2::new(0.0, 1.0),
                            size: RectSize::LockedWight(
                                0.345
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::ScoreBar
                )
            )
        );
        ui_elements.insert(
            UIElementType::RedFlagMark,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::TopLeft,
                            position: Vec2::new(0.04, 1.0),
                            size: RectSize::LockedWight(
                                0.03
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::RedFlagMark
                )
            )
        );
        ui_elements.insert(
            UIElementType::BlueFlagMark,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::TopRight,
                            position: Vec2::new(-0.04, 1.0),
                            size: RectSize::LockedWight(
                                0.03
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::BlueFlagMark
                )
            )
        );
        ui_elements.insert(
            UIElementType::FirstScoreMarkRed,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::TopLeft,
                            position: Vec2::new(-0.1, 1.0),
                            size: RectSize::LockedWight(
                                0.015
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::RedScoreMark
                )
            )
        );
        ui_elements.insert(
            UIElementType::SecondScoreMarkRed,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::TopLeft,
                            position: Vec2::new(-0.18, 1.0),
                            size: RectSize::LockedWight(
                                0.015
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::RedScoreMark
                )
            )
        );
        ui_elements.insert(
            UIElementType::ThirdScoreMarkRed,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::TopLeft,
                            position: Vec2::new(-0.26, 1.0),
                            size: RectSize::LockedWight(
                                0.015
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::RedScoreMark
                )
            )
        );
        ui_elements.insert(
            UIElementType::FinalScoreMarkRed,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::TopLeft,
                            position: Vec2::new(-0.3, 1.0),
                            size: RectSize::LockedWight(
                                0.025
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::RedFinalMark
                )
            )
        );
        ui_elements.insert(
            UIElementType::FirstScoreMarkBlue,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::TopRight,
                            position: Vec2::new(0.1, 1.0),
                            size: RectSize::LockedWight(
                                0.015
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::BlueScoreMark
                )
            )
        );
        ui_elements.insert(
            UIElementType::SecondScoreMarkBlue,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::TopRight,
                            position: Vec2::new(0.18, 1.0),
                            size: RectSize::LockedWight(
                                0.015
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::BlueScoreMark
                )
            )
        );
        ui_elements.insert(
            UIElementType::ThirdScoreMarkBlue,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::TopRight,
                            position: Vec2::new(0.26, 1.0),
                            size: RectSize::LockedWight(
                                0.015
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::BlueScoreMark
                )
            )
        );
        ui_elements.insert(
            UIElementType::FinalScoreMarkBlue,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::TopRight,
                            position: Vec2::new(0.3, 1.0),
                            size: RectSize::LockedWight(
                                0.025
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::BlueFinalMark
                )
            )
        );
        ui_elements.insert(
            UIElementType::RedTeamBacklight,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterCenter,
                            position: Vec2::new(0.0, 0.0),
                            size: RectSize::LockedBoth(
                                1.0,
                                1.0
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::BacklightRed
                )
            )
        );
        ui_elements.insert(
            UIElementType::BlueTeamBacklight,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterCenter,
                            position: Vec2::new(0.0, 0.0),
                            size: RectSize::LockedBoth(
                                1.0,
                                1.0
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::BacklightBlue
                )
            )
        );
        ui_elements.insert(
            UIElementType::RedTeamWinTitle,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterCenter,
                            position: Vec2::new(0.0, 0.0),
                            size: RectSize::LockedWight(
                                0.3,
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::RedTeamWinTitle
                )
            )
        );
        ui_elements.insert(
            UIElementType::BlueTeamWinTitle,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterCenter,
                            position: Vec2::new(0.0, 0.0),
                            size: RectSize::LockedWight(
                                0.3,
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::BlueTeamWinTitle
                )
            )
        );
        ui_elements.insert(
            UIElementType::JoinRedTeamTitle,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterCenter,
                            position: Vec2::new(0.0, 0.0),
                            size: RectSize::LockedWight(
                                0.45,
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::JoinRedTeamTitle
                )
            )
        );
        ui_elements.insert(
            UIElementType::JoinBlueTeamTitle,
            UIElement::Image(
                UIImage::new(
                    UIData::new(
                        UIRect {
                            anchor: RectAnchor::CenterCenter,
                            position: Vec2::new(0.0, 0.0),
                            size: RectSize::LockedWight(
                                0.45,
                            ),
                            rotation_around_rect_center: 0.0,
                            rotation_around_screen_center: 0.0,
                            transparency: 1.0,
                            drawing_order: 0,
                            transform_buffer: None,
                        },
                        false,
                        None,
                    ),
                    TextureType::JoinBlueTeamTitle
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


    pub fn get_mut_ui_element(
        &mut self,
        element: &UIElementType
    ) -> &mut UIElement {
        self.ui_elements
            .get_mut(element)
            .expect("Some concrete UI element is not exist")
    }

    pub fn get_ui_element(
        &self,
        element: &UIElementType
    ) -> &UIElement {
        self.ui_elements
            .get(element)
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
                    if elem.ui_data.parent_ui_elem.is_none() {
                        continue;
                    }
                    
                    let parent_transform = {
                        match self.get_ui_element(elem.ui_data.parent_ui_elem.as_ref().unwrap()) {
                            UIElement::Image(elem) => {
                                elem.ui_data.rect.get_rect_transform_uniform(
                                    elem.texture_aspect.unwrap(),
                                    screen_aspect,
                                    None,
                                )
                            },
                            UIElement::ProgressBar(elem) => {
                                elem.ui_data.rect.get_rect_transform_uniform(
                                    elem.texture_aspect.unwrap(),
                                    screen_aspect,
                                    None,
                                )
                            },
                            UIElement::ScannerDisplay(elem) => {
                                elem.ui_data.rect.get_rect_transform_uniform(
                                    1.0,
                                    screen_aspect,
                                    None,
                                )
                            }
                        }
                    };

                    queue.write_buffer(
                        elem.ui_data.rect.transform_buffer
                            .as_ref()
                            .expect("UI Image have not rect transform buffer"),
                        0,
                        bytemuck::cast_slice(&[
                            elem.ui_data.rect
                                .get_rect_transform_uniform(
                                    elem
                                        .texture_aspect
                                        .expect("UI Image have not texture aspect"),
                                    screen_aspect,
                                    Some(parent_transform),
                        )]),
                    );
                }
                UIElement::ProgressBar(elem) => {
                    if elem.ui_data.parent_ui_elem.is_none() {
                        continue;
                    }

                    let parent_transform = {
                        match self.get_ui_element(elem.ui_data.parent_ui_elem.as_ref().unwrap()) {
                            UIElement::Image(elem) => {
                                elem.ui_data.rect.get_rect_transform_uniform(
                                    elem.texture_aspect.unwrap(),
                                    screen_aspect,
                                    None,
                                )
                            },
                            UIElement::ProgressBar(elem) => {
                                elem.ui_data.rect.get_rect_transform_uniform(
                                    elem.texture_aspect.unwrap(),
                                    screen_aspect,
                                    None,
                                )
                            },
                            UIElement::ScannerDisplay(elem) => {
                                elem.ui_data.rect.get_rect_transform_uniform(
                                    1.0,
                                    screen_aspect,
                                    None,
                                )
                            }
                        }
                    };

                    queue.write_buffer(
                        elem.ui_data.rect.transform_buffer
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
                                    Some(parent_transform),
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
                },
                UIElement::ScannerDisplay(elem) => {
                    if elem.ui_data.parent_ui_elem.is_none() {
                        continue;
                    }
                    
                    let parent_transform = {
                        match self.get_ui_element(elem.ui_data.parent_ui_elem.as_ref().unwrap()) {
                            UIElement::Image(elem) => {
                                elem.ui_data.rect.get_rect_transform_uniform(
                                    elem.texture_aspect.unwrap(),
                                    screen_aspect,
                                    None,
                                )
                            },
                            UIElement::ProgressBar(elem) => {
                                elem.ui_data.rect.get_rect_transform_uniform(
                                    elem.texture_aspect.unwrap(),
                                    screen_aspect,
                                    None,
                                )
                            },
                            UIElement::ScannerDisplay(elem) => {
                                elem.ui_data.rect.get_rect_transform_uniform(
                                    1.0,
                                    screen_aspect,
                                    None,
                                )
                            }
                        }
                    };

                    queue.write_buffer(
                        elem.ui_data.rect.transform_buffer
                            .as_ref()
                            .expect("UI Image have not rect transform buffer"),
                        0,
                        bytemuck::cast_slice(&[
                            elem.ui_data.rect
                                .get_rect_transform_uniform(
                                    1.0,
                                    screen_aspect,
                                    Some(parent_transform),
                        )]),
                    );
                }
            }
        }

        for (_, ui_elem) in &self.ui_elements {
            match ui_elem {
                UIElement::Image(elem) => {
                    if elem.ui_data.parent_ui_elem.is_some() {
                        continue;
                    }
                    queue.write_buffer(
                        elem.ui_data.rect.transform_buffer
                            .as_ref()
                            .expect("UI Image have not rect transform buffer"),
                        0,
                        bytemuck::cast_slice(&[
                            elem.ui_data.rect
                                .get_rect_transform_uniform(
                                    elem
                                        .texture_aspect
                                        .expect("UI Image have not texture aspect"),
                                    screen_aspect,
                                    None,
                        )]),
                    );
                }
                UIElement::ProgressBar(elem) => {
                    if elem.ui_data.parent_ui_elem.is_some() {
                        continue;
                    }
                    queue.write_buffer(
                        elem.ui_data.rect.transform_buffer
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
                                    None,
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
                },
                UIElement::ScannerDisplay(elem) => {
                    if elem.ui_data.parent_ui_elem.is_some() {
                        continue;
                    }
                    queue.write_buffer(
                        &elem.ui_data.rect.transform_buffer
                            .as_ref()
                            .expect("UI Image have not rect transform buffer"),
                        0,
                        bytemuck::cast_slice(&[
                            elem.ui_data.rect
                                .get_rect_transform_uniform(
                                    1.0,
                                    screen_aspect,
                                    None,
                        )]),
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

    pub transparency: f32,
    pub drawing_order: usize,

    pub transform_buffer: Option<Buffer>,
}

impl UIRect {
    pub fn get_rect_transform_uniform(
        &self,
        texture_aspect: f32,
        screen_aspect: f32,
        parent_transform: Option<RectTransformUniform>,
    ) -> RectTransformUniform {

        if let Some(parent) = parent_transform {

            let scale = {
                match self.size {
                    RectSize::LockedBoth(x, y) => {
                        [parent.scale[0] * x, parent.scale[1] * y]
                    },
                    RectSize::LockedHeight(y) => {
                        unimplemented!()
                    },
                    RectSize::LockedWight(x) => {
                        unimplemented!()
                    }
                }
            };

            let translation = {
                match self.anchor {
                    RectAnchor::CenterCenter => {
                        [
                            parent.translation[0] + self.position.x * parent.scale[0],
                            parent.translation[1] + self.position.y * parent.scale[1]
                        ]
                    }
                    RectAnchor::TopRight => {
                        unimplemented!()
                    }
                    RectAnchor::TopLeft => {
                        unimplemented!()
                    }
                    RectAnchor::CenterTop => {
                        unimplemented!()
                    }
                    RectAnchor::DownLeft => {
                        unimplemented!()
                    }
                    RectAnchor::DownRight => {
                        unimplemented!()
                    }
                    RectAnchor::CenterDown => {
                        unimplemented!()
                    }
                    RectAnchor::CenterLeft => {
                        unimplemented!()
                    }
                    RectAnchor::CenterRight => {
                        unimplemented!()
                    }
                }
            };

            return RectTransformUniform {
                rotation_around_rect_center: self.rotation_around_rect_center,
                rotation_around_screen_center: self.rotation_around_screen_center,
                transparency: self.transparency,
                empty_bytes: 0.0,
                scale,
                translation,
            };
        } else {

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
            //         | wgpu screen  |
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
            
            return RectTransformUniform {
                rotation_around_rect_center: self.rotation_around_rect_center,
                rotation_around_screen_center: self.rotation_around_screen_center,
                transparency: self.transparency,
                empty_bytes: 0.0,
                scale,
                translation,
            };
        }
    }


}

pub struct UIData {
    pub is_visible: Arc<Mutex<bool>>,
    pub rect: UIRect,
    pub need_to_redraw: bool,
    pub parent_ui_elem: Option<UIElementType>,
}

impl UIData {
    pub fn new(
        rect: UIRect,
        is_visible: bool,
        parent_ui_elem: Option<UIElementType>,
    ) -> Self {
        let is_visible =  Arc::new(Mutex::new(is_visible));

        UIData {
            is_visible,
            rect,
            need_to_redraw: true,
            parent_ui_elem,
        }
    }

    pub fn initialize(&mut self, rect_transform_buffer: Buffer) {
        self.rect.transform_buffer = Some(rect_transform_buffer);
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
        self.texture_aspect = Some(texture_aspect);
        self.texture_size = Some(texture_size);

        self.ui_data.initialize(rect_transform_buffer);
    }

    pub fn set_is_visible(&mut self, is_visible: bool) {
        self.ui_data.set_is_visible(is_visible);
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
        self.progress_bar_value_buffer = Some(progress_bar_value_buffer);

        self.ui_data.initialize(rect_transform_buffer);
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
}


pub enum ScannerDisplayPlaneOrientation {
    ZX,
    ZW,
}

pub struct UIScannerDisplay {
    pub ui_data: UIData,
    orientation: ScannerDisplayPlaneOrientation,

    scanner_data_buffer: Option<Buffer>,
}

impl UIScannerDisplay {

    pub fn new(
        ui_data: UIData,
        orientation: ScannerDisplayPlaneOrientation,
    ) -> Self {

        UIScannerDisplay {
            ui_data,
            orientation,
            scanner_data_buffer: None
        }
    }

    pub fn initialize(
        &mut self,
        rect_transform_buffer: Buffer,
        scanner_data_buffer: Buffer
    ) {
        self.ui_data.initialize(rect_transform_buffer);

        self.scanner_data_buffer = Some(scanner_data_buffer);
    }

    pub fn get_scanner_data_uniform(&self) -> ScannerDataUniform {
        match self.orientation {
            ScannerDisplayPlaneOrientation::ZX => {
                ScannerDataUniform {
                    empty_byte0: 0u32,
                    empty_byte1: 0u32,
                    empty_byte2: 0u32,

                    orientation: 0u32,
                }
            },
            ScannerDisplayPlaneOrientation::ZW => {
                ScannerDataUniform {
                    empty_byte0: 0u32,
                    empty_byte1: 0u32,
                    empty_byte2: 0u32,
                    
                    orientation: 1u32,
                }
            }
        }
    }
}