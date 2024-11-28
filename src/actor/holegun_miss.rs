use glam::{Vec3, Vec4};

use crate::{
    actor::{
        Actor,
        ActorID,
    },
    engine::{
        audio::AudioSystem, engine_handle::{
            Command,
            CommandType,
            EngineHandle
        }, physics::{
            colliders_container::PhysicalElement,
            PhysicsSystem
        }, render::VisualElement, time::TimeSystem, ui::UISystem, world::static_object::{
            BeamVolumeArea,
            VolumeArea
        }
    },
    transform::Transform,
};

use super::{CommonActorsMessages, Message, MessageType};

const EXPLODE_TIME: f32 = 0.25;

pub struct HoleGunMiss {
    id: Option<ActorID>,
    transform: Transform,
    volume_areas: Vec<VolumeArea>,

    explode_current_time: f32,
    explode_final_time: f32,
    target_size: f32,
    target_size_reached: bool
}


impl HoleGunMiss {
    pub fn new(
        position: Vec4,
        shooted_from: Vec4,
        radius: f32,
        color: Vec3,
        mut charging_volume_area: VolumeArea,
        beam_radius_mult: f32,
    ) -> Self {

        let transform = Transform::from_position(position);

        match &mut charging_volume_area {
            VolumeArea::SphericalVolumeArea(area) => {
                area.translation = shooted_from - position;
            }
            _ => {
                panic!("charging volume area in HolrGun Hole is not SphericalVolumeArea")
            }
        }

        let beam = VolumeArea::BeamVolumeArea(
            BeamVolumeArea {
                translation_pos_1: Vec4::ZERO,
                translation_pos_2: shooted_from - position,
                radius: 0.020 * beam_radius_mult,
                color: color, 
            }
        );

        let mut volume_areas = Vec::with_capacity(2);

        volume_areas.push(beam);
        volume_areas.push(charging_volume_area);

        HoleGunMiss {
            id: None,
            transform,
            volume_areas,
            target_size: radius,
            target_size_reached: false,
            explode_current_time: 0.0,
            explode_final_time: EXPLODE_TIME * (radius*0.3),
        }
    }


    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform
    }
}

impl Actor for HoleGunMiss {
    fn get_id(&self) -> Option<ActorID> {
        self.id
    }

    fn change_id(&mut self, id: ActorID, engine_handle: &mut EngineHandle) {
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

    fn set_id(&mut self, id: ActorID) {
        self.id = Some(id);
    }

    fn tick(
        &mut self,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        time_system: &mut TimeSystem,
        delta: f32
    ) {
        if self.target_size_reached {
            for volume_area in self.volume_areas.iter_mut() {
                
                match volume_area {
                    VolumeArea::BeamVolumeArea(area) => {
                        area.radius *= 1.0 - delta*30.0;
                    },
                    VolumeArea::SphericalVolumeArea(area) => {
                        area.radius *= 1.0 - delta*30.0;
                        
                        if area.radius < 0.01 {
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
            }
        } else {
            self.explode_current_time += delta;

            match &mut self.volume_areas[0] {
                VolumeArea::BeamVolumeArea(area) => {
                    area.radius += delta*0.2;
                },
                _ => {}
            }
            
            match &mut self.volume_areas[1] {
                VolumeArea::SphericalVolumeArea(area) => {
                    area.radius -= delta*0.35;
                }
                _ => {}
            }

            if self.explode_current_time >= self.explode_final_time {
                self.target_size_reached = true;
            }
        }
    }

    fn get_visual_element(&self) -> Option<VisualElement> {
        Some(
            VisualElement {
                transform: &self.transform,
                static_objects:  None,
                coloring_areas: None,
                volume_areas: Some(&self.volume_areas),
                player: None,
            }
        )
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement> {
        None
    } 
}