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

use glam::{Vec3, Vec4};
use rand::Rng;

use crate::{
    actor::trgger_orb::TriggerOrbMessage, engine::{
        audio::AudioSystem, effects::EffectsSystem, engine_handle::EngineHandle, physics::{PhysicsSystem, area::{Area, AreaMessage}, colliders_container::PhysicalElement, physics_system_data::ShapeType}, render::VisualElement, time::TimeSystem, ui::UISystem, world::static_object::{self, ColoringArea, SphericalVolumeArea, StaticObject, VolumeArea}
    }, transform::Transform
};

use super::{
    Actor, ActorID, Message, MessageType, PhysicsMessages, SpecificActorMessage
};

#[derive(Clone)]
pub enum TriggerMessage
{
    ActorEnteredTriggerArea(
        // trigger name
        String,
        // trigger id
        u128,
        // players id
        u128,
        // trigger position
        Vec4,
    ),
    Triggered(
       // trigger name
       String,
    )
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

pub struct Trigger
{
    transform: Transform,
    id: Option<ActorID>,
    area: Area,
    visual_areas: Vec<VolumeArea>,
    coloring_areas: Vec<ColoringArea>,
    static_objects: Vec<StaticObject>,
    trigger_name: String,
    is_triggered: bool,
}

impl Trigger
{
    pub fn new(
        transform: Transform,
        trigger_name: String,
        trigger_area_radius: f32,
        visual_area_radius: f32,
        visual_area_color: Vec3,
        visual_area_position: Vec4,
        coloring_area_radius: f32,
        coloring_area_color: Vec3,
        coloring_area_position: Vec4,
        static_objects: Vec<StaticObject>,
    ) -> Self
    {

        let area: Area = Area::new(
            Vec4::ZERO,
            ShapeType::Sphere,
            Vec4::new(
                trigger_area_radius,
                0.0, 0.0, 0.0
            )
        );

        let mut visual_areas = Vec::with_capacity(1);

        let visual_area =  VolumeArea::SphericalVolumeArea(
            SphericalVolumeArea {
                radius: visual_area_radius,
                translation: visual_area_position,
                color: visual_area_color,
            }
        );

        visual_areas.push(visual_area);

        let mut coloring_areas = Vec::with_capacity(1);

        let coloring_area =  ColoringArea {
            radius: coloring_area_radius,
            translation: coloring_area_position,
            color: coloring_area_color,
        };

        coloring_areas.push(coloring_area);


        Trigger {
            transform,
            id: None,
            area,
            visual_areas,
            coloring_areas,
            static_objects,
            trigger_name,
            is_triggered: false,
        }
    }
}

impl Actor for Trigger
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
                static_objects: Some(&mut self.static_objects),
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
                static_objects: Some(&self.static_objects),
                coloring_areas: Some(&self.coloring_areas),
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
                                if !self.is_triggered
                                {
                                    engine_handle.send_boardcast_message(
                                        Message {
                                            from: self.id.expect("Flag have not ActorID"),
                                            remote_sender: false,
                                            message: MessageType::SpecificActorMessage(
                                                SpecificActorMessage::TriggerMessage(
                                                    TriggerMessage::ActorEnteredTriggerArea(
                                                        self.trigger_name.clone(),
                                                        self.id.expect("Flag have not ActorID"),
                                                        id,
                                                        self.transform.get_position()
                                                    )
                                                )
                                            )
                                        }
                                    );
                                }
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

            MessageType::SpecificActorMessage(message) =>
            {
                match message {
                    SpecificActorMessage::TriggerOrbMessage(message) =>
                    {
                        match message {
                            TriggerOrbMessage::TriggerOrbReachedTheTrigger =>
                            {
                                if !self.is_triggered
                                {
                                    self.is_triggered = true;

                                    engine_handle.send_boardcast_message(
                                        Message {
                                            from: self.id.expect("Flag have not ActorID"),
                                            remote_sender: false,
                                            message: MessageType::SpecificActorMessage(
                                                SpecificActorMessage::TriggerMessage(
                                                    TriggerMessage::Triggered(
                                                        self.trigger_name.clone()
                                                    )
                                                )
                                            )
                                        }
                                    );

                                }
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

