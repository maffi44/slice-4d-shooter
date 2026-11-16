// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use crate::engine::world::WorldContext;
use crate::engine::shared_systems::EngineSharedSystems;

use std::f32::consts::PI;

use glam::{Vec3, Vec4};

use crate::{
    actor::trgger_orb::TriggerOrbMessage, engine::{
        world::physics::{area::{Area, AreaMessage}, colliders_container::PhysicalElement, physics_system_data::ShapeType}, shared_systems::render::VisualElement, world::static_object::{ColoringArea, SphericalVolumeArea, StaticObject, VolumeArea}
    }, engine::transform::Transform
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
    pulse_timer: f32,
    orginal_visual_area_radius: f32,
    orginal_visual_area_color: Vec3,
    orginal_coloring_area_radius: f32,
    orginal_coloring_area_color: Vec3,
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
            pulse_timer: 0.0,
            orginal_visual_area_radius: visual_area_radius,
            orginal_visual_area_color: visual_area_color,
            orginal_coloring_area_radius: coloring_area_radius,
            orginal_coloring_area_color: coloring_area_color,
        }
    }
}

const VISUAL_AREA_SIZE_PULSE_MULT: f32 = 0.2;
const VISUAL_AREA_COLOR_PULSE_MULT: f32 = 0.01;
const COLORING_AREA_SIZE_PULSE_MULT: f32 = 0.285;
const COLORING_AREA_COLOR_PULSE_MULT: f32 = 0.208;

impl Actor for Trigger
{
    fn tick(
        &mut self,
        world: &mut WorldContext,
        _shared: &mut EngineSharedSystems,
    )
    {
        let delta = world.time.delta();

        self.pulse_timer += delta; 
        if self.pulse_timer >= PI {self.pulse_timer -= PI}

        let pulse = f32::sin(self.pulse_timer)*0.5+0.5;
        let visual_area_size_pulse_val = pulse*(1.0-VISUAL_AREA_SIZE_PULSE_MULT)+VISUAL_AREA_SIZE_PULSE_MULT;
        let visual_area_color_pulse_val = pulse*(1.0-VISUAL_AREA_COLOR_PULSE_MULT)+VISUAL_AREA_COLOR_PULSE_MULT;
        let coloring_area_size_pulse_val = pulse*(1.0-COLORING_AREA_SIZE_PULSE_MULT)+COLORING_AREA_SIZE_PULSE_MULT;
        let coloring_area_color_pulse_val = pulse*(1.0-COLORING_AREA_COLOR_PULSE_MULT)+COLORING_AREA_COLOR_PULSE_MULT;

        if let VolumeArea::SphericalVolumeArea(area) = &mut self.visual_areas[0] {
            area.radius = self.orginal_visual_area_radius * visual_area_size_pulse_val;
            area.color = self.orginal_visual_area_color * visual_area_color_pulse_val;
        }
        self.coloring_areas[0].radius = self.orginal_coloring_area_radius * coloring_area_size_pulse_val;
        self.coloring_areas[0].color = self.orginal_coloring_area_color * coloring_area_color_pulse_val;
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
        world: &mut WorldContext,
        _shared: &mut EngineSharedSystems,
    ) {
        let messenger = &mut *world.messenger;

        
        let _from = message.from;

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
                                    messenger.send_broadcast_message(
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
                    SpecificActorMessage::TriggerOrbMessage(message) =>
                    {
                        match message {
                            TriggerOrbMessage::TriggerOrbReachedTheTrigger =>
                            {
                                if !self.is_triggered
                                {
                                    self.is_triggered = true;

                                    messenger.send_broadcast_message(
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

