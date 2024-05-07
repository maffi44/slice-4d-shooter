use glam::{Vec3, Vec4, FloatExt};

use crate::{
    actor::{
        Actor,
        ActorID,
    },
    engine::{
        engine_handle::{
            Command,
            CommandType,
            EngineHandle
        },
        physics::{
            colliders_container::PhysicalElement,
            physics_system_data::ShapeType,
            static_collider::StaticCollider,
            PhysicsSystem
        },
        render::VisualElement,
        world::static_object::{
            BeamVolumeArea,
            ColoringArea,
            ObjectMaterial,
            SphericalVolumeArea,
            StaticObject,
            VolumeArea
        },
    },
    transform::Transform,
};

use super::{CommonActorsMessages, Component, Message, MessageType};

const SHOT_HOLE_START_RADIUS: f32 = 0.01;
const SHOT_HOLE_FINAL_RADIUS: f32 = 0.11;
const SHOT_HOLE_GROWING_SPEED: f32 = 1.5;
const SHOT_HOLE_REDUCTION_SPEED: f32 = 0.5;

const SHOT_EXPLODE_START_RADIUS: f32 = 0.34;
const SHOT_EXPLODE_FINAL_RADIUS: f32 = 0.6;
const SHOT_EXPLODE_GROWNIG_SPEED: f32 = 3.0;

const GUN_EXPLODE_START_RADIUS: f32 = 0.11;
const GUN_EXPLODE_FINAL_RADIUS: f32 = 0.15;
const GUN_EXPLODE_GROWNIG_SPEED: f32 = 3.0;

const BEAM_START_RADIUS: f32 = 0.02;
const BEAM_FINAL_RADIUS: f32 = 0.06;
const BEAM_GROWING_SPEED: f32 = 3.0;

const COLOR: Vec3 = Vec3::new(1.0, 0.0, 0.0);

pub struct MachinegunShot {
    id: Option<ActorID>,
    transform: Transform,
    static_objects: Vec<StaticObject>,
    coloring_areas: Vec<ColoringArea>,
    volume_areas: Vec<VolumeArea>,

    hole_target_size_reached: bool,
    it_is_miss: bool,
}


impl MachinegunShot {
    pub fn new(
        position: Vec4,
        shooted_from: Vec4,
        beam_radius_mult: f32,
        shot_explode_radius_mult: f32,
        it_is_miss: bool
    ) -> Self {

        let transform = Transform::from_position(position);

        let mut static_objects = Vec::with_capacity(1);

        let mut coloring_areas = Vec::with_capacity(1);

        let mut volume_areas = Vec::with_capacity(3);

        let charging_volume_area = VolumeArea::SphericalVolumeArea(
            SphericalVolumeArea {
                translation: shooted_from - position,
                radius: GUN_EXPLODE_START_RADIUS * shot_explode_radius_mult,
                color: COLOR,
            }
        );

        let beam = VolumeArea::BeamVolumeArea(
            BeamVolumeArea {
                translation_pos_1: Vec4::ZERO,
                translation_pos_2: shooted_from - position,
                radius: BEAM_START_RADIUS * beam_radius_mult,
                color: COLOR, 
            }
        );

        volume_areas.push(beam);
        volume_areas.push(charging_volume_area);


        if !it_is_miss {
            let static_object = StaticObject {
                collider: StaticCollider {
                    shape_type: ShapeType::Sphere,
                    position: Vec4::ZERO,
                    size: Vec4::new(SHOT_HOLE_START_RADIUS, 0.0, 0.0, 0.0),
                    is_positive: false,
                    roundness: 0.0,
                    stickiness: false,
                    friction: 0.0,
                    bounce_rate: 0.0,
                    actors_id: None,
                },
                material_index: -1,
            };
    
            static_objects.push(static_object);
            
            let coloring_area = ColoringArea {
                translation: Vec4::ZERO,
                radius: SHOT_HOLE_START_RADIUS * 1.3,
                color: COLOR
            };

            coloring_areas.push(coloring_area);

            let explode = VolumeArea::SphericalVolumeArea(
                SphericalVolumeArea {
                    translation: Vec4::ZERO,
                    radius: SHOT_EXPLODE_START_RADIUS,
                    color: COLOR, 
                }
            );
    
            volume_areas.push(explode);
        }

        MachinegunShot {
            id: None,
            transform,
            static_objects,
            coloring_areas,
            volume_areas,
            hole_target_size_reached: false,
            it_is_miss,
        }
    }


    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform
    }
}

impl Actor for MachinegunShot {
    fn get_id(&self) -> Option<ActorID> {
        self.id
    }

    fn set_id(&mut self, id: ActorID, engine_handle: &mut EngineHandle) {
        if let Some(prev_id) = self.id {
            engine_handle.send_boardcast_message(Message {
                from: prev_id,
                message: MessageType::CommonActorsMessages(
                    CommonActorsMessages::IWasChangedMyId(
                        id
                    )
                )
            });
        }

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
        if self.hole_target_size_reached {
            
            for area in self.coloring_areas.iter_mut() {
                area.radius -= delta * SHOT_HOLE_REDUCTION_SPEED;
            }
            
            for obj in self.static_objects.iter_mut() {
                obj.collider.size.x -= delta * SHOT_HOLE_REDUCTION_SPEED;

                if obj.collider.size.x <= 0.01 {
                    
                    engine_handle.send_command(
                        Command {
                            sender: self.id.expect("MachinegunShot have not ActorID"),
                            command_type: CommandType::RemoveActor(
                                self.id.expect("MachinegunShot have not ActorID")
                            )
                        }
                    );

                    return;
                }
            }
        } else {

            for area in self.coloring_areas.iter_mut() {
                area.radius += delta * SHOT_HOLE_GROWING_SPEED;
            }
            
            for obj in self.static_objects.iter_mut() {
                obj.collider.size.x += delta * SHOT_HOLE_GROWING_SPEED;

                if obj.collider.size.x >= SHOT_HOLE_FINAL_RADIUS {
                    self.hole_target_size_reached = true;
                }
            }
        }

        if !self.volume_areas.is_empty() {
            
            let mut size_reached = false;
            
            match &mut self.volume_areas[0] {
                VolumeArea::BeamVolumeArea(area) => {
                    area.radius += delta*BEAM_GROWING_SPEED;
                },
                _ => {}
            }

            match &mut self.volume_areas[1] {
                VolumeArea::SphericalVolumeArea(area) => {
                    area.radius += delta * GUN_EXPLODE_GROWNIG_SPEED;

                    if area.radius >= GUN_EXPLODE_FINAL_RADIUS {
                        size_reached = true;
                    }
                }
                _ => {}
            }

            if !self.it_is_miss {

                match &mut self.volume_areas[2] {
                    VolumeArea::SphericalVolumeArea(area) => {
                        area.radius += delta*SHOT_EXPLODE_GROWNIG_SPEED;
                    }
                    _ => {}
                }

            }

            if size_reached {self.volume_areas.clear()}
        }
    }

    fn get_visual_element(&self) -> Option<VisualElement> {
        Some(
            VisualElement {
                transform: &self.transform,
                static_objects:  Some(&self.static_objects),
                coloring_areas: Some(&self.coloring_areas),
                volume_areas: Some(&self.volume_areas),
                player: None,
            }
        )
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement> {
        Some(
            PhysicalElement {
                transform: &mut self.transform,
                static_objects: Some(&mut self.static_objects),
                dynamic_colliders: None,
                kinematic_collider: None,
                static_colliders: None,
                area: None,
            }
        )
    } 
}