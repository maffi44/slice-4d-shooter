use std::collections::{HashMap, HashSet, VecDeque};

use glam::{Vec3, Vec4};

use crate::{engine::{physics::{area::{Area, AreaMessage}, colliders_container::PhysicalElement, physics_system_data::ShapeType}, render::VisualElement, world::static_object::{SphericalVolumeArea, VolumeArea}}, transform::Transform};

use super::{Actor, ActorID, Message, MessageType, PhysicsMessages, SpecificActorMessage};


const TIME_TO_NOT_INTERACT_WITH_ACTOR: f32 = 1.0;
const MOVER_W_PHYSICAL_AREA_RADIUS: f32 = 1.0;

#[derive(Clone)]
pub enum MoverWMessage
{
    Rotate,
}

pub struct MoverW
{
    transform: Transform,
    id: Option<ActorID>,
    actors_to_not_interact_with: HashMap<ActorID, f32>,
    actors_to_remove_from_list: Vec<ActorID>,
    physical_area: Area,
    volume_areas: Vec<VolumeArea>,
}

impl MoverW
{
    pub fn new(
        position: Vec4,
    ) -> Self
    {
        let physical_area = Area::new(
            Vec4::ZERO,
            ShapeType::Sphere,
            Vec4::new(MOVER_W_PHYSICAL_AREA_RADIUS, 0.0, 0.0, 0.0)
        );

        let visual_area = VolumeArea::SphericalVolumeArea(
            SphericalVolumeArea {
                translation: Vec4::ZERO,
                radius: MOVER_W_PHYSICAL_AREA_RADIUS,
                color: Vec3::new(0.8,0.3,0.8),
            }
        );

        let volume_areas = vec![visual_area];

        MoverW {
            transform: Transform::from_position(position),
            id: None,
            actors_to_not_interact_with: HashMap::new(),
            actors_to_remove_from_list: Vec::new(),
            physical_area,
            volume_areas,
        }
    }
}

impl Actor for MoverW
{
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
        
        for (id, time) in self.actors_to_not_interact_with.iter_mut()
        {
            *time -= delta;

            if *time < 0.0
            {
                self.actors_to_remove_from_list.push(*id);
            }
        }

        while let Some(id) = self.actors_to_remove_from_list.pop() {
            self.actors_to_not_interact_with.remove(&id);
        }
    }

    fn recieve_message(
        &mut self,
        message: super::Message,
        engine_handle: &mut crate::engine::engine_handle::EngineHandle,
        physics_system: &crate::engine::physics::PhysicsSystem,
        audio_system: &mut crate::engine::audio::AudioSystem,
        ui_system: &mut crate::engine::ui::UISystem,
        time_system: &crate::engine::time::TimeSystem,
        effects_system: &mut crate::engine::effects::EffectsSystem,
    )
    {
        let from = message.from;
        match message.message {
            MessageType::PhysicsMessages(message) =>
            {
                match message {
                    PhysicsMessages::AreaMessage(message) =>
                    {
                        match message
                        {
                            AreaMessage::ActorEnterArea(id) =>
                            {
                                if !self.actors_to_not_interact_with.contains_key(&id)
                                {
                                    self.actors_to_not_interact_with.insert(
                                        id,
                                        TIME_TO_NOT_INTERACT_WITH_ACTOR
                                    );

                                    engine_handle.send_direct_message(
                                        id,
                                        Message {
                                            from: self.id.expect("MoverW hasn't ActorID"),
                                            message: MessageType::SpecificActorMessage(
                                                SpecificActorMessage::MoverW(
                                                    MoverWMessage::Rotate
                                                )
                                            )
                                        }
                                    );
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

    fn get_physical_element(&mut self) -> Option<PhysicalElement> {
        Some(
            PhysicalElement {
                id: self.id.expect("MoverW have not ActorID"),
                transform: &mut self.transform,
                kinematic_collider: None,
                dynamic_colliders: None,
                static_colliders: None,
                static_objects: None,
                area: Some(&mut self.physical_area),
            }
        )
    }

    fn get_visual_element(&self) -> Option<VisualElement> {
        Some(
            VisualElement {
                transform: &self.transform,
                static_objects: None,
                coloring_areas: None,
                volume_areas: Some(&self.volume_areas),
                waves: None,
                player: None
            }
        )
    }
}