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

use std::f32::consts::PI;

use fyrox_sound::source::Status;
use glam::{Vec3, Vec4};

use crate::{
    engine::{
        audio::{AudioSystem, Sound}, effects::EffectsSystem, engine_handle::{Command, CommandType, EngineHandle}, physics::{PhysicsSystem, area::{Area, AreaMessage}, colliders_container::PhysicalElement, physics_system_data::ShapeType, static_collider::StaticCollider}, render::VisualElement, time::TimeSystem, ui::UISystem, world::static_object::{ColoringArea, StaticObject}
    }, transform::Transform
};

use super::{
     Actor, ActorID, CommonActorsMessage, Message, MessageType, PhysicsMessages, SpecificActorMessage
};

#[derive(Clone)]
pub enum DropedRotatorToolMessage
{
    DropedRotatorToolCapturedByPlayer,
    YouInteractingWithDropedRotatorTool,
}


pub struct DropedRotatorTool
{
    transform: Transform,
    id: Option<ActorID>,
    area: Area,
    static_objects: Vec<StaticObject>,
    coloring_areas: Vec<ColoringArea>,
    pulse_timer: f32,
}

impl DropedRotatorTool
{
    pub fn new(
        transform: Transform,
    ) -> Self
    {

        let area: Area = Area::new(
            Vec4::ZERO,
            ShapeType::Sphere,
            Vec4::new(
                0.5,
                0.0, 0.0, 0.0
            )
        );

        let sphere = StaticObject {
                collider: StaticCollider {
                position: Vec4::ZERO,
                size: Vec4::ONE,
                is_positive: true,
                roundness: 0.0,
                stickiness: false,
                friction: 0.0,
                bounce_rate: 0.0,
                shape_type: ShapeType::Sphere,
                undestroyable: true,
                actor_id: None,
            },
            material_index: 3,
        };

        let static_objects = vec![sphere];

        let coloring_area =  ColoringArea {
            radius: 1.8,
            translation: Vec4::ZERO,
            color: Vec3::Y,
        };

        let coloring_areas = vec![coloring_area];

        DropedRotatorTool {
            transform,
            id: None,
            area,
            static_objects,
            coloring_areas,
            pulse_timer: 0.0,
        }
    }
}

impl Actor for DropedRotatorTool
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
        self.pulse_timer += delta*2.0; 
        if self.pulse_timer >= PI*2.0 {self.pulse_timer -= PI*2.0}
        let pulse = f32::sin(self.pulse_timer);
        self.transform.increment_position(Vec4::Y*pulse*0.5*delta);
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
                static_objects: Some(&self.static_objects),
                coloring_areas: Some(&self.coloring_areas),
                volume_areas: None,
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
            MessageType::SpecificActorMessage(message) =>
            {
                match message {
                    SpecificActorMessage::DropedRotatorToolMessage(message) =>
                    {
                        match message
                        {
                            DropedRotatorToolMessage::DropedRotatorToolCapturedByPlayer =>
                            {
                                audio_system.spawn_spatial_sound(
                                    Sound::GetScore,
                                    1.0,
                                    1.0,
                                    false,
                                    true,
                                    Status::Playing,
                                    self.transform.get_position(),
                                    1.0,
                                    1.0,
                                    15.0
                                );

                                engine_handle.send_command(
                                    Command {
                                        sender: self.get_id().expect("Droped Rotator Tool havn't ActorID"),
                                        command_type: CommandType::RemoveActor(
                                            self.get_id().expect("Droped Rotator Tool havn't ActorID")
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
                                            SpecificActorMessage::DropedRotatorToolMessage(
                                                DropedRotatorToolMessage::YouInteractingWithDropedRotatorTool
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

