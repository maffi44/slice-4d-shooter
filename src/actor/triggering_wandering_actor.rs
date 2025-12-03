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

use crate::{
    actor::{Message, MessageType, SpecificActorMessage, trigger::TriggerMessage, wandering_actor::WanderingActorMovementType}, engine::{
        audio::AudioSystem, effects::EffectsSystem, engine_handle::EngineHandle, physics::{
            PhysicsSystem, colliders_container::PhysicalElement
        }, render::VisualElement, time::TimeSystem, ui::UISystem, world::static_object::StaticObject
    }, transform::Transform
};

use std::f32::consts::PI;

use super::{Actor, ActorID};

pub struct TriggeringWanderingActor {
    transform: Transform,
    id: Option<ActorID>,
    static_objects: Vec<StaticObject>,

    triggering_by_trigger: String,
    is_one_time_movement: bool,
    triggered: bool,
    first_path_completed: bool,

    movement_type: WanderingActorMovementType,
    targets: [Transform; 2],
    travel_time: f32,
    current_target_index: usize,
    current_travel_time: f32,
}

impl TriggeringWanderingActor {
    pub fn new(
        transform: Transform,
        target: Transform,
        triggering_by_trigger: String,
        is_one_time_movement: bool,
        static_objects: Vec<StaticObject>,
        travel_time: f32,
        movement_type: WanderingActorMovementType,
    ) -> Self {
        let first_target = transform.clone();

        let targets = [first_target, target];

        let current_target = 1_usize;

        let current_travel_time = 0_f32;

        TriggeringWanderingActor {
            transform,
            static_objects,
            travel_time,
            movement_type,
            is_one_time_movement,
            triggering_by_trigger,
            triggered: false,
            first_path_completed: false,
            targets,
            current_target_index: current_target,
            current_travel_time,
            id: None
        }
    }
}

const THRESHOLD: f32 = 0.01;

impl Actor for TriggeringWanderingActor {
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

    fn get_physical_element(&mut self) -> Option<PhysicalElement> {
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

    fn get_visual_element(&self) -> Option<VisualElement> {
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
        if self.triggered
        {
            if self.is_one_time_movement && self.first_path_completed
            {
                return;
            }

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

                self.first_path_completed = true;
                // change target
                // change 0 to 1 or 1 to 0
                self.current_target_index = (self.current_target_index as i32 - 1 as i32).abs() as usize;
    
                self.current_travel_time = 0.0;
    
            } else {
                self.current_travel_time += delta;
            };
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