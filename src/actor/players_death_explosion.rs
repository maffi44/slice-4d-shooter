use glam::{Vec3, Vec4};

use crate::{engine::{audio::AudioSystem, engine_handle::{Command, CommandType, EngineHandle}, physics::{physics_system_data::ShapeType, static_collider::StaticCollider}, render::VisualElement, world::static_object::{ColoringArea, SphericalVolumeArea, StaticObject, VolumeArea}}, transform::Transform};

use super::{Actor, ActorID};

pub struct PlayersDeathExplosion {
    id: Option<ActorID>,
    transform: Transform,
    volume_areas: Vec<VolumeArea>,
    static_objects: Vec<StaticObject>,
    coloring_areas: Vec<ColoringArea>,
    hole_target_size_reached: bool,


}

impl PlayersDeathExplosion {
    pub fn new(position: Vec4) -> Self {
        let mut volume_areas = Vec::with_capacity(1);

        let volume_area = VolumeArea::SphericalVolumeArea(
            SphericalVolumeArea {
                translation: Vec4::ZERO,
                radius: 0.6,
                color: Vec3::new(13.0, 3.0, 0.0),
            }
        );

        volume_areas.push(volume_area);

        let static_object = StaticObject {
            collider: StaticCollider {
                shape_type: ShapeType::Sphere,
                position: Vec4::ZERO,
                size: Vec4::new(0.01, 0.0, 0.0, 0.0),
                is_positive: false,
                roundness: 0.0,
                stickiness: false,
                friction: 0.0,
                bounce_rate: 0.0,
                actors_id: None,
            },
            material_index: -1,
        };

        let mut static_objects = Vec::with_capacity(1);

        static_objects.push(static_object);

        let coloring_area = ColoringArea {
            translation: Vec4::ZERO,
            radius: 0.15,
            color: Vec3::new(5.0, 1.0, 0.0)
        };

        let mut coloring_areas = Vec::with_capacity(1);

        coloring_areas.push(coloring_area);


        PlayersDeathExplosion {
            id: None,
            transform: Transform::from_position(position),
            volume_areas,
            static_objects,
            coloring_areas,
            hole_target_size_reached: false,
        }
    }
}

impl Actor for PlayersDeathExplosion {
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
                static_objects: Some(&self.static_objects),
                coloring_areas: Some(&self.coloring_areas),
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
        if self.hole_target_size_reached {
    
            for obj in self.static_objects.iter_mut() {
                obj.collider.size.x -= delta * 0.2;
    
                if obj.collider.size.x <= 0.0 {
                    engine_handle.send_command(
                        Command {
                            sender: self.id.expect("player death explosion have not ActorID"),
                            command_type: CommandType::RemoveActor(
                                self.id.expect("player death explosion have not ActorID")
                            )
                        }
                    )
                }
            }
            for coloring_area in self.coloring_areas.iter_mut() {
                coloring_area.radius -= delta * 0.2;
            }
        } else {
            for obj in self.static_objects.iter_mut() {
                obj.collider.size.x  += delta*12.0;
            }
            for coloring_area in self.coloring_areas.iter_mut() {
                coloring_area.radius += delta*12.0;
            }
        }

        for volume_area in self.volume_areas.iter_mut() {

            if let VolumeArea::SphericalVolumeArea(area) = volume_area {
                area.radius += delta*22.0;

                if area.radius > 3.9 {
                    let my_id = self.id.expect("PlayerDeathExplode have not ActorID");
                    
                    self.hole_target_size_reached = true;
                    
                }
            }
        }
        
        if self.hole_target_size_reached {
            self.volume_areas.clear();
        }
    }
}