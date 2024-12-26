use std::collections::{HashMap, HashSet, VecDeque};

use glam::{Vec3, Vec4};

use crate::{engine::{physics::{area::{Area, AreaMessage}, colliders_container::PhysicalElement, physics_system_data::ShapeType}, render::VisualElement, world::static_object::{SphericalVolumeArea, VolumeArea}}, transform::Transform};

use super::{player::{BLUE_TEAM_COLOR, RED_TEAM_COLOR}, Actor, ActorID, Message, MessageType, PhysicsMessages, SpecificActorMessage};


const TIME_TO_NOT_INTERACT_WITH_ACTOR: f32 = 1.0;
const MOVER_W_PHYSICAL_AREA_RADIUS: f32 = 0.8;
const MOVER_W_VISUAL_AREA_RADIUS: f32 = 0.5;

#[derive(Clone)]
pub enum MoverWMessage
{
    Rotate(
        // z_lock
        f32,
        // w_lock
        f32,
        // direction
        f32
    ),
}

const VISUAL_WAVE_TICK_TIME: f32 = 2.2;
const MOVER_W_COLOR: Vec3 = Vec3::new(0.5,0.14,0.5);

pub struct MoverW
{
    transform: Transform,
    id: Option<ActorID>,
    direction: f32,
    actors_to_not_interact_with: HashMap<ActorID, f32>,
    actors_to_remove_from_list: Vec<ActorID>,
    physical_area: Area,
    volume_areas: Vec<VolumeArea>,
    vlisual_wave_tick_timer: f32,
    base_color: Vec3,
}

impl MoverW
{
    pub fn new(
        position: Vec4,
        mut direction: f32,
        w_levels: &Vec<f32>,
    ) -> Self
    {
        assert!(w_levels.len() > 1);

        let base_coef = 
        {
            let w_pos = position.w;

            f32::clamp(
                (w_pos - w_levels[0]) /
                (*w_levels.last().unwrap() - w_levels[0]),
                    0.0,
                    1.0
            )
        };

        let base_color = BLUE_TEAM_COLOR.lerp(RED_TEAM_COLOR, base_coef);
        
        if direction < 0.0
        {
            direction = -1.0;
        }
        else
        {
            direction = 1.0;    
        }

        let physical_area = Area::new(
            Vec4::ZERO,
            ShapeType::Sphere,
            Vec4::new(MOVER_W_PHYSICAL_AREA_RADIUS, 0.0, 0.0, 0.0)
        );

        let visual_area = VolumeArea::SphericalVolumeArea(
            SphericalVolumeArea {
                translation: Vec4::ZERO,
                radius: MOVER_W_PHYSICAL_AREA_RADIUS,
                color: MOVER_W_COLOR,
            }
        );

        let volume_areas = vec![visual_area];

        MoverW {
            transform: Transform::from_position(position),
            id: None,
            direction,
            actors_to_not_interact_with: HashMap::new(),
            actors_to_remove_from_list: Vec::new(),
            physical_area,
            volume_areas,
            vlisual_wave_tick_timer: 0.0,
            base_color,
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

        self.vlisual_wave_tick_timer += delta;

        if self.vlisual_wave_tick_timer >= VISUAL_WAVE_TICK_TIME
        {
            self.vlisual_wave_tick_timer = 0.0;

            effects_system.spawn_wave(
                engine_handle,
                self.transform.get_position(),
                vec![3.6, 1.8, 0.0],
                vec![Vec3::ZERO, self.base_color*0.012, MOVER_W_COLOR*0.8],
                vec![0.29, 0.29],
            );

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
                                                    MoverWMessage::Rotate(
                                                        self.transform.get_position().z,
                                                        self.transform.get_position().w,
                                                        self.direction
                                                    )
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