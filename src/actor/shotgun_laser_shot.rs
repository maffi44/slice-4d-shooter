use glam::Vec4;

use crate::transform::Transform;

use super::{Actor, ActorID};



pub struct ShotgunLaserShot
{
    transform: Transform,
    id: Option<ActorID>,
    real_start_position: Vec4,
    visible_start_position: Vec4,
    possible_destination: Vec4,
    damage_dealer_id: ActorID,
}

impl ShotgunLaserShot
{
    pub fn new(
        real_start_position: Vec4,
        visible_start_position: Vec4,
        possible_destination: Vec4,
        damage_dealer_id: ActorID,

    ) -> Self
    {
        let transform = Transform::from_position(visible_start_position);

        ShotgunLaserShot
        {
            transform,
            id: None,
            real_start_position,
            visible_start_position,
            possible_destination,
            damage_dealer_id,
        }
    }
}

impl Actor for ShotgunLaserShot
{
    fn get_mut_transform(&mut self) -> &mut Transform {
        &mut self.transform
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

    fn tick(
        &mut self,
        physic_system: &crate::engine::physics::PhysicsSystem,
        engine_handle: &mut crate::engine::engine_handle::EngineHandle,
        audio_system: &mut crate::engine::audio::AudioSystem,
        ui_system: &mut crate::engine::ui::UISystem,
        time_system: &mut crate::engine::time::TimeSystem,
        effects_system: &mut crate::engine::effects::EffectsSystem,
        delta: f32
    ) {
        
    }
}