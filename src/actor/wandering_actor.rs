use crate::{
    engine::{
        world::static_object::StaticObject,
        engine_handle::EngineHandle,
        render::VisualElement,
        physics::{
            PhysicsSystem,
            colliders_container::PhysicalElement,
        },
    },
    transform::Transform
};

use std::f32::consts::PI;

use super::{Actor, ActorID, Component};

pub enum WonderingActorMovementType {
    Linear,
    NonLinear,
}

pub struct WanderingActor {
    transform: Transform,
    id: Option<ActorID>,
    static_objects: Vec<StaticObject>,

    movement_type: WonderingActorMovementType,
    targets: [Transform; 2],
    travel_time: f32,
    current_target_index: usize,
    current_travel_time: f32,
}

impl WanderingActor {
    pub fn new(
        transform: Transform,
        static_objects: Vec<StaticObject>,
        second_target: Transform,
        travel_time: f32,
        movement_type: WonderingActorMovementType,
    ) -> Self {
        let first_target = transform.clone();

        let targets = [first_target, second_target];

        let current_target = 1_usize;

        let current_travel_time = 0_f32;

        WanderingActor {
            transform,
            static_objects,
            travel_time,
            movement_type,
            targets,
            current_target_index: current_target,
            current_travel_time,
            id: None
        }
    }
}

const THRESHOLD: f32 = 0.01;

impl Actor for WanderingActor {
    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_mut_transform(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn get_id(&self) -> Option<ActorID> {
        self.id
    }

    fn init(&mut self, id: ActorID) {
        self.id = Some(id);

        for static_object in self.static_objects.iter_mut() {
            static_object.collider.init(id);
        }
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement> {
        let physical_element = PhysicalElement {
            transform: &mut self.transform,
            kinematic_collider: None,
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
        };

        Some(visual_element)
    }

    
    fn tick(
        &mut self,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        delta: f32
    ) {    
        let current_target = self.targets[self.current_target_index];

        let previous_target = self.targets[
            // current_target always is 0 or 1.
            // if current_target is 0 we get 1
            // and if current_target is 1 we get 0
            (self.current_target_index as i32 - 1 as i32).abs() as usize
        ];
        
        match self.movement_type {
            WonderingActorMovementType::Linear => {
                
                let mut movement_speed = current_target.get_position() - previous_target.get_position();
        
                let mut scaling_speed = current_target.get_scale() - previous_target.get_scale();

                movement_speed /= self.travel_time;
                
                scaling_speed /= self.travel_time;

                self.transform.increment_position(movement_speed * delta);
                self.transform.increment_scale(scaling_speed * delta);

            },
            WonderingActorMovementType::NonLinear => {
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
            // change target
            // change 0 to 1 or 1 to 0
            self.current_target_index = (self.current_target_index as i32 - 1 as i32).abs() as usize;

            self.current_travel_time = 0.0;

        } else {
            self.current_travel_time += delta;
        };
    }
}