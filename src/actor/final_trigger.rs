// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use crate::engine::shared_systems::resources::{ResourceRead, ResourceRequests, Retention};
use fyrox_sound::buffer::SoundBufferResource;
use crate::engine::world::WorldContext;
use crate::engine::shared_systems::EngineSharedSystems;
use crate::engine::world::effects;

use std::f32::consts::PI;

use fyrox_resource::Resource;
use crate::engine::shared_systems::resources::WorldResource;

use crate::engine::world::OuterWorldMessage;

use fyrox_sound::source::Status;
use glam::{Vec3, Vec4};

use crate::{
    actor::{trgger_orb::TriggerOrbMessage, trigger::TriggerMessage}, engine::{
        world::physics::{area::{Area, AreaMessage}, colliders_container::PhysicalElement, physics_system_data::ShapeType}, shared_systems::render::VisualElement, world::static_object::{ColoringArea, SphericalVolumeArea, StaticObject, VolumeArea}
    }, engine::transform::Transform
};

use super::{
    Actor, ActorID, Message, MessageType, PhysicsMessages, SpecificActorMessage
};

#[derive(Clone)]
pub enum FinalTrggerMessage
{
    PlayerBeatLevel
}

pub struct FinalTrgger
{
    team_win_sound: Option<SoundBufferResource>,


    transform: Transform,
    id: Option<ActorID>,
    area: Area,
    visual_areas: Vec<VolumeArea>,
    coloring_areas: Vec<ColoringArea>,
    static_objects: Vec<StaticObject>,
    is_triggered: bool,
    level_transition_timer: f32,
    next_level_name: String,
    async_preload: bool,
    transition_finished: bool,
    /// The next world's descriptor, loading in the background (preload).
    next_world: Option<Resource<WorldResource>>,
    pulse_timer: f32,
    orginal_visual_area_radius: f32,
    orginal_visual_area_color: Vec3,
    orginal_coloring_area_radius: f32,
    orginal_coloring_area_color: Vec3,
}

impl FinalTrgger
{
    fn start_loading_next_world(&mut self, shared: &mut EngineSharedSystems)
    {
        self.next_world = Some(shared.resources.request_world(&self.next_level_name));
    }

    pub fn new(
        resources: &mut ResourceRequests,
        transform: Transform,
        next_level_name: String,
        async_preload: bool,
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

        FinalTrgger {
            team_win_sound: resources.sound(
                crate::embedded_asset!("sounds/team_win.wav"), Retention::Session,
            ),

            transform,
            next_level_name,
            async_preload,
            transition_finished: false,
            next_world: None,
            id: None,
            area,
            visual_areas,
            coloring_areas,
            static_objects,
            is_triggered: false,
            level_transition_timer: 0.0,
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
const COLORING_AREA_SIZE_PULSE_MULT: f32 = 0.485;
const COLORING_AREA_COLOR_PULSE_MULT: f32 = 0.208;

impl Actor for FinalTrgger
{
    fn on_added_to_world(&mut self, _: &mut WorldContext, shared: &mut EngineSharedSystems) {
        if self.async_preload {
            self.start_loading_next_world(shared);
        }
    }

    fn tick(
        &mut self,
        world: &mut WorldContext,
        shared: &mut EngineSharedSystems,
    )
    {
        let delta = world.time.delta();
        let messenger = &mut *world.messenger;

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


        if self.is_triggered
        {
            self.level_transition_timer += delta;

            if self.level_transition_timer > 3.0
            {
                messenger.send_broadcast_message(
                    Message {
                        from: self.id.expect("Flag have not ActorID"),
                        remote_sender: false,
                        message: MessageType::SpecificActorMessage(
                            SpecificActorMessage::FinalTrggerMessage(
                                FinalTrggerMessage::PlayerBeatLevel
                            )
                        )
                    }
                );
            }

            if self.level_transition_timer > 5.0 && !self.transition_finished
            {
                let handle = self.next_world.get_or_insert_with(|| shared.resources.request_world(&self.next_level_name));
                match handle.try_read(|world| world.0.as_ref().expect("Loaded world descriptor").clone())
                {
                    std::task::Poll::Pending => return,
                    std::task::Poll::Ready(Ok(descriptor)) => {
                        messenger.send_to_outer_world(OuterWorldMessage::ReplaceRootWorld(descriptor));
                    }
                    std::task::Poll::Ready(Err(error)) => {
                        log::error!("Could not load world '{}': {error:?}", self.next_level_name);
                    }
                }
                self.transition_finished = true;
                self.next_world = None;
            }
        }
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
        message: &Message,
        world: &mut WorldContext,
        shared: &mut EngineSharedSystems,
    ) {
        let messenger = &mut *world.messenger;
        let audio_system = &mut shared.audio;

        
        let _from = message.from;

        match &message.message
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
                                                        "final_trigger".to_string(),
                                                        self.id.expect("Flag have not ActorID"),
                                                        *id,
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

                                    audio_system.spawn_spatial_sound(
                                        &self.team_win_sound,
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

                                    effects::spawn_wave(
                                        messenger,
                                        self.transform.get_position(),
                                        vec![
                                            0.0,
                                            15.0,
                                        ],
                                        vec![
                                            Vec3::ONE,
                                            Vec3::ZERO
                                        ],
                                        vec![
                                            2.0,
                                        ]
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

