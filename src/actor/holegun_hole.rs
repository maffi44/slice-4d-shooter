use glam::{Vec3, Vec4};

use crate::{
    actor::{
        Actor,
        ActorID,
    },
    engine::{
        engine_handle::EngineHandle, physics::{colliders_container::PhysicalElement, physics_system_data::ShapeType, static_collider::StaticCollider, PhysicsSystem}, render::VisualElement, world::static_object::{self, ObjectMatrial, StaticObject}
    },
    transform::{self, Transform},
};

use super::Component;



pub struct HoleGunHole {
    id: Option<ActorID>,
    transform: Transform,
    static_objects: Vec<StaticObject>,
}


impl HoleGunHole {
    pub fn new() -> Self {
        let static_object = StaticObject {
            collider: StaticCollider {
                shape_type: ShapeType::Sphere,
                position: Vec4::ZERO,
                size: Vec4::new(2.0, 0.0, 0.0, 0.0),
                is_positive: false,
                roundness: 0.0,
                stickiness: false,
                friction: 0.0,
                bounce_rate: 0.0,
                actors_id: None,
            },
            material: ObjectMatrial {
                color: Vec3::new(0.0, 1.0, 0.0),
            }
        };

        let mut static_objects = Vec::with_capacity(1);

        static_objects.push(static_object);

        HoleGunHole {
            id: None,
            transform: Transform::new_zero(),
            static_objects,
        }
    }


    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform
    }
}

impl Actor for HoleGunHole {
    fn get_id(&self) -> Option<ActorID> {
        self.id
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_mut_transform(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn init(&mut self, id: ActorID) {
        self.id = Some(id);

        for static_object in self.static_objects.iter_mut() {
            static_object.collider.init(id);
        }
    }

    fn tick(
        &mut self,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        delta: f32
    ) {

    }

    fn get_visual_element(&self) -> Option<VisualElement> {
        Some(
            VisualElement {
                transfrom: &self.transform,
                static_objects: &self.static_objects
            }
        )
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement> {
        Some(
            PhysicalElement {
                transform: &mut self.transform,
                static_objects: Some(&mut self.static_objects),
                kinematic_collider: None,
                static_colliders: None,
                area: None,
            }
        )
    } 
}