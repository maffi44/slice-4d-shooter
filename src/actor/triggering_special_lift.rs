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

use fyrox_core::math::lerpf;
use glam::Vec4;

use crate::{
    actor::{Message, MessageType, SpecificActorMessage, trigger::TriggerMessage, wandering_actor::WanderingActorMovementType}, engine::{
        audio::AudioSystem, effects::EffectsSystem, engine_handle::EngineHandle, physics::{
            PhysicsSystem, colliders_container::PhysicalElement
        }, render::VisualElement, time::TimeSystem, ui::UISystem, world::static_object::StaticObject
    }, transform::Transform
};

use std::f32::consts::PI;

use super::{Actor, ActorID};

pub struct TriggeringSpecialLiftActor {
    transform: Transform,
    id: Option<ActorID>,
    static_objects: Vec<StaticObject>,

    lift_door_index: usize,
    lift_door_max_size: Vec4,
    lift_door_max_roundness: f32,
    lift_door_open_time: f32,

    triggering_by_trigger: String,
    triggered: bool,

    movement_type: WanderingActorMovementType,
    targets: [Transform; 2],
    travel_time: f32,
    current_target_index: usize,
    current_travel_time: f32,
    is_stopped: bool,
    current_stop_time: f32,
    stop_time: f32,
    pulse_timer: f32,
}

impl TriggeringSpecialLiftActor {
    pub fn new(
        transform: Transform,
        target: Transform,
        triggering_by_trigger: String,
        mut static_objects: Vec<StaticObject>,
        lift_door_index: usize,
        lift_door_open_time: f32,
        travel_time: f32,
        stop_time: f32,
        movement_type: WanderingActorMovementType,
    ) -> Self {
        let first_target = transform.clone();

        let targets = [first_target, target];

        let current_target = 1_usize;

        let current_travel_time = 0_f32;

        let lift_door_max_size = static_objects[lift_door_index].collider.size;
        let lift_door_max_roundness = static_objects[lift_door_index].collider.roundness;

        static_objects[lift_door_index].collider.size = Vec4::ZERO;
        static_objects[lift_door_index].collider.roundness = 0.0;

        TriggeringSpecialLiftActor {
            transform,
            static_objects,
            lift_door_index,
            lift_door_max_size,
            lift_door_max_roundness,
            lift_door_open_time,
            travel_time,
            movement_type,
            triggering_by_trigger,
            triggered: false,
            targets,
            current_target_index: current_target,
            current_travel_time,
            is_stopped: true,
            id: None,
            current_stop_time: 0.0,
            stop_time,
            pulse_timer: 0.0,
        }
    }
}

const THRESHOLD: f32 = 0.01;
const PULSE_MULT: f32 = 0.12;

impl Actor for TriggeringSpecialLiftActor {
    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_mut_transform(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn get_id(&self) -> Option<ActorID> {
        self.id
    }

    fn set_id(&mut self, id: ActorID) {
        self.id = Some(id);
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement<'_>> {
        let physical_element = PhysicalElement {
            id: self.get_id().expect("Actor have not ActorID"),
            transform: &mut self.transform,
            kinematic_collider: None,
            dynamic_colliders: None,
            static_colliders: None,
            static_objects: Some(&mut self.static_objects),
            area: None
        };

        Some(physical_element)
    }

    fn get_visual_element(&self) -> Option<VisualElement<'_>> {
        let visual_element = VisualElement {
            transform: &self.transform,
            static_objects: Some(&self.static_objects),
            coloring_areas: None,
            volume_areas: None,
            waves: None,
            player: None,
            child_visual_elem: None,
        };

        Some(visual_element)
    }

    
    fn tick(
        &mut self,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        time_system: &mut TimeSystem,
        effects_system: &mut EffectsSystem,
        delta: f32
    ) {

        self.pulse_timer += delta*2.0; 
        if self.pulse_timer >= PI*2.0 {self.pulse_timer -= PI*2.0}
        let pulse = f32::sin(self.pulse_timer);
        self.transform.increment_position(Vec4::Y*pulse*PULSE_MULT*delta);

        if self.triggered
        {
            if self.is_stopped
            {
                self.current_stop_time += delta;

                if self.current_stop_time < self.stop_time/2.0
                {
                    let size = Vec4::lerp(
                        Vec4::ZERO,
                        self.lift_door_max_size,
                        self.current_stop_time.min(self.lift_door_open_time) /
                        self.lift_door_open_time
                    );

                    let roundness = lerpf(
                        0.0,
                        self.lift_door_max_roundness,
                        self.current_stop_time.min(self.lift_door_open_time) /
                        self.lift_door_open_time
                    );

                    self.static_objects[self.lift_door_index].collider.size = size;
                    self.static_objects[self.lift_door_index].collider.roundness = roundness;
                }
                else
                {
                    let size = Vec4::lerp(
                        Vec4::ZERO,
                        self.lift_door_max_size,
                        (self.stop_time - self.current_stop_time)  
                        .min(self.lift_door_open_time) /
                        self.lift_door_open_time
                    );

                    let roundness = lerpf(
                        0.0,
                        self.lift_door_max_roundness,
                        (self.stop_time - self.current_stop_time)  
                        .min(self.lift_door_open_time) /
                        self.lift_door_open_time
                    );

                    self.static_objects[self.lift_door_index].collider.size = size;
                    self.static_objects[self.lift_door_index].collider.roundness = roundness;
                }

                if self.current_stop_time >= self.stop_time
                {
                    self.is_stopped = false;
                    self.current_stop_time = 0.0;
                }
            }
            else
            {
                let current_target = self.targets[self.current_target_index];
        
                let previous_target = self.targets[
                    // current_target always is 0 or 1.
                    // if current_target is 0 we get 1
                    // and if current_target is 1 we get 0
                    (self.current_target_index as i32 - 1 as i32).abs() as usize
                ];
                
                match self.movement_type {
                    WanderingActorMovementType::Linear => {
                        
                        let mut movement_speed = current_target.get_position() - previous_target.get_position();
                
                        let mut scaling_speed = current_target.get_scale() - previous_target.get_scale();
        
                        movement_speed /= self.travel_time;
                        
                        scaling_speed /= self.travel_time;
        
                        self.transform.increment_position(movement_speed * delta);
                        self.transform.increment_scale(scaling_speed * delta);
        
                    },
                    WanderingActorMovementType::NonLinear => {
                        let coefficient = {
                            f32::sin((self.current_travel_time / self.travel_time) * (PI/2.0))
                        };
        
                        let new_position = previous_target.get_position().lerp(
                            current_target.get_position(),
                            coefficient
                        );
        
                        let new_scale = previous_target.get_scale().lerp(
                            current_target.get_scale(),
                            coefficient
                        );
        
                        self.transform.set_position(new_position);
                        self.transform.set_scale(new_scale);
                    }
                }
        
                if self.current_travel_time >= self.travel_time {
    
                    self.is_stopped = true;
                    self.current_stop_time = 0.0;
    
                    // change target
                    // change 0 to 1 or 1 to 0
                    self.current_target_index = (self.current_target_index as i32 - 1 as i32).abs() as usize;
        
                    self.current_travel_time = 0.0;
        
                } else {
                    self.current_travel_time += delta;
                };
            }
        }
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

        match message.message {
            MessageType::SpecificActorMessage(message) =>
            {
                match message {
                    SpecificActorMessage::TriggerMessage(message) =>
                    {
                        match message {
                            TriggerMessage::Triggered(trigger_name) =>
                            {
                                if self.triggering_by_trigger == trigger_name
                                {
                                    self.triggered = true;
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