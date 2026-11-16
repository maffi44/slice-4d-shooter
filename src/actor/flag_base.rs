// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use crate::engine::world::WorldContext;
use crate::engine::shared_systems::EngineSharedSystems;

use client_server_protocol::Team;
use glam::Vec4;

use crate::{
    actor::session_controller::SessionControllerMessage, engine::{
        world::physics::{area::{Area, AreaMessage}, colliders_container::PhysicalElement, physics_system_data::ShapeType}, shared_systems::render::VisualElement, world::static_object::{SphericalVolumeArea, VolumeArea}
    }, engine::transform::Transform
};

use super::{
    main_player::{BLUE_TEAM_COLOR, RED_TEAM_COLOR}, Actor, ActorID, Message, MessageType, PhysicsMessages, SpecificActorMessage
};

#[derive(Clone)]
pub enum FlagBaseMessage
{
    YouInteractingWithFlagBase(Team),
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
        }
    }
}

impl Actor for FlagBase
{
    fn tick(
        &mut self,
        _world: &mut WorldContext,
        _shared: &mut EngineSharedSystems,
    )
    {


    }

    fn get_mut_transform(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement<'_>>
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


    fn get_visual_element(&self) -> Option<VisualElement<'_>>
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
        world: &mut WorldContext,
        _shared: &mut EngineSharedSystems,
    ) {
        let messenger = &mut *world.messenger;

        
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
                                messenger.send_direct_message(
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
                            AreaMessage::ActorIsContainedInsideArea(_id) =>
                            {

                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            MessageType::SpecificActorMessage(message) =>
            {
                match message {
                    SpecificActorMessage::SessionControllerMessage(message) =>
                    {
                        match message {
                            SessionControllerMessage::GiveMeFlagsPostitions =>
                            {
                                messenger.send_direct_message(
                                    from,
                                    Message {
                                        from: self.id.unwrap(),
                                        remote_sender: false,
                                        message: MessageType::SpecificActorMessage(
                                            SpecificActorMessage::SessionControllerMessage(
                                                SessionControllerMessage::ItIsFlagPosition(
                                                    self.owned_by_team,
                                                    self.transform.get_position()
                                                )
                                            )
                                        )
                                    }
                                );
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

