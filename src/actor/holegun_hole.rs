use glam::{Vec3, Vec4};
use serde_json::de;

use crate::{
    actor::{
        Actor,
        ActorID,
    },
    engine::{
        engine_handle::{Command, CommandType, EngineHandle}, physics::{colliders_container::PhysicalElement, physics_system_data::ShapeType, static_collider::StaticCollider, PhysicsSystem}, render::VisualElement, world::static_object::{self, BeamVolumeArea, ColoringArea, ObjectMatrial, SphericalVolumeArea, StaticObject, VolumeArea}
    },
    transform::{self, Transform},
};

use super::Component;

const HOLE_COLOR: Vec3 = Vec3::new(0.2, 1.0, 0.0);

pub struct HoleGunHole {
    id: Option<ActorID>,
    transform: Transform,
    static_objects: Vec<StaticObject>,
    coloring_areas: Vec<ColoringArea>,
    volume_areas: Vec<VolumeArea>,
}


impl HoleGunHole {
    pub fn new(position: Vec4, shoooted_from: Vec4, radius: f32, color: Vec3) -> Self {

        let transform = Transform::new_from_pos(position);

        let static_object = StaticObject {
            collider: StaticCollider {
                shape_type: ShapeType::Sphere,
                position: Vec4::ZERO,
                size: Vec4::new(radius, 0.0, 0.0, 0.0),
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

        let coloring_area = ColoringArea {
            translation: Vec4::ZERO,
            radius: radius + 0.1,
            color: color
        };

        let mut coloring_areas = Vec::with_capacity(1);

        coloring_areas.push(coloring_area);

        let beam = VolumeArea::BeamVolumeArea(
            BeamVolumeArea {
                translation_pos_1: Vec4::ZERO,
                translation_pos_2: shoooted_from - position,
                radius: 0.015 + radius*0.1,
                color: color, 
            }
        );

        let explode = VolumeArea::SphericalVolumeArea(
            SphericalVolumeArea {
                translation: Vec4::ZERO,
                radius: radius*1.2,
                color: color, 
            }
        );

        let mut volume_areas = Vec::with_capacity(2);

        volume_areas.push(beam);
        volume_areas.push(explode);

        HoleGunHole {
            id: None,
            transform,
            static_objects,
            coloring_areas,
            volume_areas,
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
        for area in self.coloring_areas.iter_mut() {
            area.radius -= delta * 0.2;
        }

        let mut clear = false;

        for volume_area in self.volume_areas.iter_mut() {
            
            match volume_area {
                VolumeArea::BeamVolumeArea(area) => {
                    area.radius *= 1.0 - delta*30.0;

                    // if area.radius < 0.001 {
                    //     clear = true;
                    // }
                },
                VolumeArea::SphericalVolumeArea(area) => {
                    area.radius *= 1.0 - delta*30.0;
                    
                    if area.radius < 0.01 {
                        clear = true;
                    }
                }
            }
        }

        if clear {
            self.volume_areas.clear();
        }

        for obj in self.static_objects.iter_mut() {
            obj.collider.size.x -= delta * 0.2;

            if obj.collider.size.x <= 0.0 {
                engine_handle.send_command(
                    Command {
                        sender: self.id.expect("HoleGun's Hole have not ActorID"),
                        command_type: CommandType::RemoveActor(
                            self.id.expect("HoleGun's Hole have not ActorID")
                        )
                    }
                )
            }
        }
    }

    fn get_visual_element(&self) -> Option<VisualElement> {
        Some(
            VisualElement {
                transfrom: &self.transform,
                static_objects:  Some(&self.static_objects),
                coloring_areas: Some(&self.coloring_areas),
                volume_areas: Some(&self.volume_areas),
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