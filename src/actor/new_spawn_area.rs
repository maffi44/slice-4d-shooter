// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use crate::engine::world::WorldContext;
use crate::engine::shared_systems::EngineSharedSystems;

use glam::Vec4;

use crate::{
    engine::world::physics::{area::{Area, AreaMessage}, colliders_container::PhysicalElement, physics_system_data::ShapeType},
    engine::transform::Transform
};

use super::{
     Actor, ActorID, CommonActorsMessage, Message, MessageType, PhysicsMessages, SpecificActorMessage
};

#[derive(Clone)]
pub enum NewSpawnAreaMessage
{
    SetNewSpawnPosition(Vec4),
}


pub struct NewSpawnArea
{
    transform: Transform,
    id: Option<ActorID>,
    area: Area,
    new_spawn_position: Vec4,
}

impl NewSpawnArea
{
    pub fn new(
        transform: Transform,
        area_size: Vec4,
        new_spawn_position: Vec4,
    ) -> Self
    {

        let area: Area = Area::new(
            Vec4::ZERO,
            ShapeType::Cube,
            area_size,
        );

        NewSpawnArea {
            transform,
            id: None,
            area,
            new_spawn_position,
        }
    }
}

impl Actor for NewSpawnArea
{
    fn tick(
        &mut self,
        _world: &mut WorldContext,
        _shared: &mut EngineSharedSystems,
    ) {
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
        message: &Message,
        world: &mut WorldContext,
        shared: &mut EngineSharedSystems,
    ) {
        let messenger = &mut *world.messenger;
        let _audio_system = &mut shared.audio;

        
        let _from = message.from;

        match &message.message
        {
            MessageType::SpecificActorMessage(_message) => {}

            MessageType::CommonActorsMessages(message) =>
            {
                match message {
                    CommonActorsMessage::SetTransform(tr) =>
                    {
                        self.transform = *tr;
                    }
                    CommonActorsMessage::ClientDisconnectedFromGameServer =>
                    {
                        // self.set_flag_on_base_status(effects_system, audio_system, messenger);
                    }
                    _ => {}
                }
            }
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
                                    *id,
                                    Message {
                                        from: self.get_id().expect("Droped Rotator Tool havn't ActorID"),
                                        remote_sender: false,
                                        message: MessageType::SpecificActorMessage(
                                            SpecificActorMessage::NewSpawnArea(
                                                NewSpawnAreaMessage::SetNewSpawnPosition(self.new_spawn_position)
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
        }
        
    }
}

