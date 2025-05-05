use crate::{engine::{audio::AudioSystem, effects::EffectsSystem, engine_handle::{Command, CommandType, EngineHandle}, physics::{colliders_container::PhysicalElement, physics_system_data::ShapeType, static_collider::StaticCollider, PhysicsSystem}, render::VisualElement, time::TimeSystem, ui::UISystem, world::static_object::{ColoringArea, StaticObject}}, transform::Transform};

use super::{Actor, ActorID};

use glam::{FloatExt, Vec3, Vec4};



pub struct Hole
{
    transform: Transform,
    id: Option<ActorID>,
    target_size: f32,
    target_size_reached: bool,
    explode_current_time: f32,
    explode_final_time: f32,
    color: Vec3,
    coloring_areas: Vec<ColoringArea>,
    static_objects: Vec<StaticObject>,
}

impl Hole
{
    pub fn new(
        transform: Transform,
        current_radius: f32,
        color: Vec3,
        target_size_reached: bool,
        target_radius: f32,
        explode_current_time: f32,
        explode_final_time: f32,
    ) -> Self
    {
        let static_object = StaticObject {
            collider: StaticCollider {
                shape_type: ShapeType::Sphere,
                position: Vec4::ZERO,
                size: Vec4::new(current_radius, 0.0, 0.0, 0.0),
                is_positive: false,
                roundness: 0.0,
                stickiness: false,
                friction: 0.0,
                bounce_rate: 0.0,
                actors_id: None,
                undestroyable: false,
            },
            material_index: -1,
        };

        let mut static_objects = Vec::with_capacity(1);

        static_objects.push(static_object);

        let coloring_area = ColoringArea {
            translation: Vec4::ZERO,
            radius: current_radius * 1.1,
            color: color
        };

        let mut coloring_areas = Vec::with_capacity(1);

        coloring_areas.push(coloring_area);


        Hole
        {
            transform,
            id: None,
            target_size: target_radius,
            target_size_reached,
            explode_current_time,
            explode_final_time,
            color,
            static_objects,
            coloring_areas,
        }
    }
}

impl Actor for Hole
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
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        audio_system: &mut AudioSystem,
        ui_system: &mut UISystem,
        time_system: &mut TimeSystem,
        effects_system: &mut EffectsSystem,
        delta: f32
    ) {
        if self.target_size_reached {
            
            for area in self.coloring_areas.iter_mut() {
                area.radius -= delta * 0.2;
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
        } else {
            self.explode_current_time += delta;

            let explode_coeff = self.explode_current_time / self.explode_final_time;

            for area in self.coloring_areas.iter_mut() {
                area.radius = f32::lerp(
                    0.0,
                    self.target_size*1.1,
                    explode_coeff.clamp(0.0, 1.0)
                );
            }

            for obj in self.static_objects.iter_mut() {
                obj.collider.size.x  = f32::lerp(
                    0.0,
                    self.target_size,
                    explode_coeff.clamp(0.0, 1.0)
                );
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
                static_objects:  Some(&self.static_objects),
                coloring_areas: Some(&self.coloring_areas),
                volume_areas: None,
                player: None,
                waves: None,
                child_visual_elem: None,
            }
        )
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement> {
        Some(
            PhysicalElement
            {
                id: self.get_id().expect("Actor have not ActorID"),
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