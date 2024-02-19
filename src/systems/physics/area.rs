use crate::systems::transform::Transform;

use super::{
    kinematic_collider::KinematicCollider,
    dynamic_collider::DynamicCollider,
    static_collider::StaticCollider,
    physics_system_data::StaticCollidersData,
};

pub struct Area {
    transform: Transform,
    radius: f32,
}

impl Area {
    pub fn new(transform: Transform, radius: f32) -> Self {
        Area {
            transform,
            radius,
        }
    }

    pub fn physic_tick(
        kinematic_colliders: &Vec<KinematicCollider>,
    ) {
        for kinematic_collider in kinematic_colliders {
            let position = kinematic_collider.transform.get_position();

            
        }
    }
}