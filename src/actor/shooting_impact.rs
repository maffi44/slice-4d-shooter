use glam::{FloatExt, Vec3, Vec4};

use crate::{engine::{audio::AudioSystem, engine_handle::{Command, CommandType, EngineHandle}, render::VisualElement, world::static_object::{SphericalVolumeArea, VolumeArea}}, transform::Transform};

use super::{player::{self, Player}, Actor, ActorID};

pub struct ShootingImpact {
    id: Option<ActorID>,
    transform: Transform,
    volume_areas: Vec<VolumeArea>,
    max_radius: f32,
}

impl ShootingImpact {
    pub fn new(position: Vec4, damage: u32) -> Self {
        let mut volume_areas = Vec::with_capacity(1);

        let volume_area = VolumeArea::SphericalVolumeArea(
            SphericalVolumeArea {
                translation: Vec4::ZERO,
                radius: 0.05,
                color: Vec3::new(12.0, 4.0, 0.0),
            }
        );

        let max_radius = 0.5.lerp(
            1.5,
            damage as f32 / player::PLAYER_MAX_HP as f32
        ); 

        volume_areas.push(volume_area);

        ShootingImpact {
            id: None,
            transform: Transform::from_position(position),
            volume_areas,
            max_radius,
        }
    }
}

impl Actor for ShootingImpact {
    fn get_id(&self) -> Option<ActorID> {
        self.id
    }

    fn set_id(&mut self, id: ActorID, engine_handle: &mut EngineHandle) {
        self.id = Some(id);
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_mut_transform(&mut self) -> &mut Transform {
        &mut self.transform
    }
    
    fn init(&mut self, id: ActorID) {
        self.id = Some(id);
    }

    fn get_visual_element(&self) -> Option<VisualElement> {
        Some(
            VisualElement {
                transform: &self.transform,
                static_objects: None,
                coloring_areas: None,
                volume_areas: Some(&self.volume_areas),
                player: None,
            },
        )
    }

    fn tick(
        &mut self,
        physic_system: &crate::engine::physics::PhysicsSystem,
        engine_handle: &mut EngineHandle,
        audio_system: &mut AudioSystem,
        delta: f32
    ) {
        if let VolumeArea::SphericalVolumeArea(area) = &mut self.volume_areas[0] {
            area.radius += delta*9.0*(1.0+area.radius*3.0);

            
            if area.radius > self.max_radius {
                let my_id = self.id.expect("PlayerDeathExplode have not ActorID");
                
                engine_handle.send_command(
                    Command {
                        sender: my_id,
                        command_type: CommandType::RemoveActor(
                            my_id
                        ),
                    }
                )
            }
        }
    }
}