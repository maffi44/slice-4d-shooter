// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use crate::engine::world::WorldContext;
use crate::engine::shared_systems::EngineSharedSystems;
use crate::engine::world::effects;

use std::{f32::consts::PI, sync::Arc};

use tokio::task::JoinHandle;

use crate::engine::world::{world_descriptor::WorldDescriptor, OuterWorldMessage};

use fyrox_sound::source::Status;
use glam::{Vec3, Vec4};

use crate::{
    actor::{trgger_orb::TriggerOrbMessage, trigger::TriggerMessage}, engine::{
        shared_systems::audio::Sound, world::physics::{area::{Area, AreaMessage}, colliders_container::PhysicalElement, physics_system_data::ShapeType}, shared_systems::render::VisualElement, world::static_object::{ColoringArea, SphericalVolumeArea, StaticObject, VolumeArea}
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
    async_load: bool,
    async_load_command_sended: bool,
    /// The next world's descriptor, loading in the background (preload).
    /// Later this will be a Resource System handle.
    next_world: Option<JoinHandle<WorldDescriptor>>,
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
        let name = self.next_level_name.clone();
        let gpu = shared.gpu.clone();

        self.next_world = Some(shared.runtime.spawn(async move {
            WorldDescriptor::load(&name, gpu).await
        }));
    }

    pub fn new(
        transform: Transform,
        next_level_name: String,
        async_preload: bool,
        async_load: bool,
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
            transform,
            next_level_name,
            async_preload,
            async_load,
            async_load_command_sended: false,
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


        if self.async_preload
        {
            self.start_loading_next_world(shared);
            
            self.async_preload = false;
        }

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

            if self.level_transition_timer > 5.0 && !self.async_load_command_sended
            {
                if self.next_world.is_none()
                {
                    self.start_loading_next_world(shared);
                }

                let next_world_is_ready = self.next_world
                    .as_ref()
                    .is_some_and(|handle| handle.is_finished());

                // async: wait (keep playing) until the descriptor is ready; sync: block until it's ready
                if next_world_is_ready || !self.async_load
                {
                    let handle = self.next_world.take().unwrap();

                    let descriptor = shared.runtime
                        .block_on(handle)
                        .expect("ERROR: next world loading task failed");

                    messenger.send_to_outer_world(
                        OuterWorldMessage::ReplaceRootWorld(Arc::new(descriptor))
                    );

                    self.async_load_command_sended = true;
                }
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
        message: Message,
        world: &mut WorldContext,
        shared: &mut EngineSharedSystems,
    ) {
        let messenger = &mut *world.messenger;
        let audio_system = &mut shared.audio;

        
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
                                                        "final_trigger".to_string(),
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

                                    audio_system.spawn_spatial_sound(
                                        Sound::TeamWin,
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

