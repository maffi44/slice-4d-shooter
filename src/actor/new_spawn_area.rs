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

use glam::Vec4;

use crate::{
    engine::{
        audio::AudioSystem, effects::EffectsSystem, engine_handle::EngineHandle, physics::{PhysicsSystem, area::{Area, AreaMessage}, colliders_container::PhysicalElement, physics_system_data::ShapeType}, time::TimeSystem, ui::UISystem,
    },
    transform::Transform
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
        physic_system: &crate::engine::physics::PhysicsSystem,
        engine_handle: &mut EngineHandle,
        audio_system: &mut crate::engine::audio::AudioSystem,
        ui_system: &mut crate::engine::ui::UISystem,
        time_system: &mut crate::engine::time::TimeSystem,
        effects_system: &mut EffectsSystem,
        delta: f32
    ) {}

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
            MessageType::SpecificActorMessage(message) => {}

            MessageType::CommonActorsMessages(message) =>
            {
                match message {
                    CommonActorsMessage::SetTransform(tr) =>
                    {
                        self.transform = tr;
                    }
                    CommonActorsMessage::ClientDisconnectedFromGameServer =>
                    {
                        // self.set_flag_on_base_status(effects_system, audio_system, engine_handle);
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
                                engine_handle.send_direct_message(
                                    id,
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
                            AreaMessage::ActorIsContainedInsideArea(id) =>
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

