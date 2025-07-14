use client_server_protocol::Team;
use glam::Vec4;
use rand::Rng;

use crate::{
    engine::{
        audio::AudioSystem, effects::EffectsSystem, engine_handle::EngineHandle, physics::{area::{Area, AreaMessage}, colliders_container::PhysicalElement, physics_system_data::ShapeType, PhysicsSystem}, render::VisualElement, time::TimeSystem, ui::UISystem, world::static_object::{SphericalVolumeArea, VolumeArea}
    },
    transform::Transform
};

use super::{
    main_player::{BLUE_TEAM_COLOR, RED_TEAM_COLOR}, Actor, ActorID, Message, MessageType, PhysicsMessages, SpecificActorMessage
};

#[derive(Clone)]
pub enum FlagBaseMessage
{
    YouInteractingWithFlagBase(Team),
}

fn get_random_vec4(range_min: f32, range_max: f32) -> Vec4
{
    assert!(range_min < range_max);

    let mut rng = rand::thread_rng();
    let x = rng.gen_range(range_min..=range_max);
    let y = rng.gen_range(range_min..=range_max);
    let z = rng.gen_range(range_min..=range_max);
    let w = rng.gen_range(range_min..=range_max);

    return Vec4::new(x, y, z, w);
}

pub const FLAG_BASE_AREA_VISUAL_RADIUS: f32 = 0.7;
pub const FLAG_BASE_AREA_PHYSICAL_RADIUS: f32 = 1.5;

pub struct FlagBase
{
    transform: Transform,
    id: Option<ActorID>,
    area: Area,
    owned_by_team: Team,
    visual_areas: Vec<VolumeArea>,
    radius_mult: f32,
}

impl FlagBase
{
    pub fn new(owned_by_team: Team, transform: Transform) -> Self
    {

        let area: Area = Area::new(
            Vec4::ZERO,
            ShapeType::Sphere,
            Vec4::new(
                FLAG_BASE_AREA_PHYSICAL_RADIUS,
                0.0, 0.0, 0.0
            )
        );

        let mut visual_areas = Vec::with_capacity(1);

        let my_color = match owned_by_team
        {
            Team::Red =>
            {
                RED_TEAM_COLOR
            }
            
            Team::Blue =>
            {
                BLUE_TEAM_COLOR
            }
        };

        let visual_area =  VolumeArea::SphericalVolumeArea(
            SphericalVolumeArea {
                radius: FLAG_BASE_AREA_VISUAL_RADIUS,
                translation: Vec4::ZERO,
                color: my_color,
            }
        );

        visual_areas.push(visual_area);


        FlagBase {
            transform,
            id: None,
            owned_by_team,
            area,
            visual_areas,
            radius_mult: 1.0,
        }
    }
}

impl Actor for FlagBase
{
    fn tick(
            &mut self,
            physic_system: &crate::engine::physics::PhysicsSystem,
            engine_handle: &mut EngineHandle,
            audio_system: &mut crate::engine::audio::AudioSystem,
            ui_system: &mut crate::engine::ui::UISystem,
            time_system: &mut crate::engine::time::TimeSystem,
            effects_system: &mut EffectsSystem,
            delta: f32
        )
    {

    }

    fn get_mut_transform(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement>
    {
        Some(
            PhysicalElement
            {
                id: self.get_id().expect("Actor have not ActorID"),
                transform: &mut self.transform,
                kinematic_collider: None,
                dynamic_colliders: None,
                static_colliders: None,
                static_objects: None,
                area: Some(&mut self.area)
            }
        )
    }


    fn get_visual_element(&self) -> Option<VisualElement>
    {
        Some(
            VisualElement
            {
                transform: &self.transform,
                static_objects: None,
                coloring_areas: None,
                volume_areas: Some(&self.visual_areas),
                waves: None,
                player: None,
                child_visual_elem: None,
            }
        )
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_id(&self) -> Option<ActorID> {
        self.id
    }

    fn set_id(&mut self, id: ActorID) {
        self.id = Some(id);
    }

    fn recieve_message(
            &mut self,
            message: Message,
            engine_handle: &mut EngineHandle,
            physics_system: &PhysicsSystem,
            audio_system: &mut AudioSystem,
            ui_system: &mut UISystem,
            time_system: &TimeSystem,
            effects_system: &mut EffectsSystem,
        ) {
        
        let from = message.from;

        match message.message
        {
            MessageType::PhysicsMessages(message) =>
            {
                match message {
                    PhysicsMessages::AreaMessage(message) =>
                    {
                        match message
                        {
                            AreaMessage::ActorEnterArea(id) =>
                            {
                                engine_handle.send_direct_message(
                                    id,
                                    Message {
                                        from: self.id.expect("Flag have not ActorID"),
                                        remote_sender: false,
                                        message: MessageType::SpecificActorMessage(
                                            SpecificActorMessage::FlagBaseMessage(
                                                FlagBaseMessage::YouInteractingWithFlagBase(
                                                    self.owned_by_team,
                                                )
                                            )
                                        )
                                    }
                                );
                            }
                            AreaMessage::ActorIsContainedInsideArea(id) =>
                            {

                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            _ => {}
        }
        
    }
}

