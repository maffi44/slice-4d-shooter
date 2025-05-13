use glam::{Vec3, Vec4, FloatExt};

use crate::{
    actor::{
        Actor,
        ActorID,
    },
    engine::{
        audio::AudioSystem, effects::EffectsSystem, engine_handle::{
            Command,
            CommandType,
            EngineHandle
        }, physics::{
            colliders_container::PhysicalElement,
            physics_system_data::ShapeType,
            static_collider::StaticCollider,
            PhysicsSystem
        }, render::VisualElement, time::TimeSystem, ui::UISystem, world::static_object::{
            BeamVolumeArea, ColoringArea, SphericalVolumeArea, StaticObject, VisualWave, VolumeArea
        }
    },
    transform::Transform,
};

const EXPLODE_TIME: f32 = 0.25;

pub struct HoleGunShot {
    id: Option<ActorID>,
    transform: Transform,
    static_objects: Vec<StaticObject>,
    coloring_areas: Vec<ColoringArea>,
    volume_areas: Vec<VolumeArea>,

    explode_current_time: f32,
    explode_final_time: f32,
    target_size: f32,
    target_size_reached: bool,
    waves: Vec<VisualWave>,
}


impl HoleGunShot {
    pub fn new(
        position: Vec4,
        shooted_from: Vec4,
        radius: f32,
        color: Vec3,
        mut charging_volume_area: VolumeArea,
        beam_radius_mult: f32,
    ) -> Self {

        let transform = Transform::from_position(position);

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
                actor_id: None,
                undestroyable: false,
            },
            material_index: -1,
        };

        let mut static_objects = Vec::with_capacity(1);

        static_objects.push(static_object);

        let coloring_area = ColoringArea {
            translation: Vec4::ZERO,
            radius: 0.01,
            color: color
        };

        let mut coloring_areas = Vec::with_capacity(1);

        coloring_areas.push(coloring_area);

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
                radius: 0.020 * beam_radius_mult.abs(),
                color: color, 
            }
        );

        let explode = VolumeArea::SphericalVolumeArea(
            SphericalVolumeArea {
                translation: Vec4::ZERO,
                radius: 0.06,
                color: color,//Vec3::new(1.0, 1.0, 1.0), 
            }
        );

        let explode_wave = VisualWave {
            translation: Vec4::ZERO,
            radius: 0.06,
            color: color * 0.01,
        };

        let waves = vec![explode_wave];

        let mut volume_areas = Vec::with_capacity(3);

        volume_areas.push(beam);
        volume_areas.push(explode);
        volume_areas.push(charging_volume_area);

        HoleGunShot {
            id: None,
            transform,
            static_objects,
            coloring_areas,
            volume_areas,
            target_size: radius.abs(),
            target_size_reached: false,
            explode_current_time: 0.0,
            explode_final_time: EXPLODE_TIME * (radius.abs()*0.3),
            waves,
        }
    }


    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform
    }
}

impl Actor for HoleGunShot {
    fn get_id(&self) -> Option<ActorID> {
        self.id
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
        effects_system: &mut EffectsSystem,
        delta: f32
    ) {
        if self.target_size_reached {
            
            for area in self.coloring_areas.iter_mut() {
                area.radius -= delta * 0.2;
                area.radius = area.radius.abs();
            }
    
            let mut clear = false;

            for wave in self.waves.iter_mut()
            {
                wave.radius *= 1.0 - delta*30.0;
                wave.radius = wave.radius.abs();

            }
    
            for volume_area in self.volume_areas.iter_mut() {
                
                match volume_area {
                    VolumeArea::BeamVolumeArea(area) => {
                        area.radius *= 1.0 - delta*30.0;
    
                        if area.radius < 0.001 {
                            clear = true;
                        }
                        area.radius = area.radius.abs();

                    },
                    VolumeArea::SphericalVolumeArea(area) => {
                        area.radius *= 1.0 - delta*105.0;
                        
                        if area.radius < 0.001 {
                            clear = true;
                        }
                        area.radius = area.radius.abs();

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
        } else {
            self.explode_current_time += delta;

            let explode_coeff = self.explode_current_time / self.explode_final_time;

            for area in self.coloring_areas.iter_mut() {
                area.radius = f32::lerp(
                    0.0,
                    self.target_size*1.1,
                    explode_coeff.clamp(0.0, 1.0)
                );
                area.radius = area.radius.abs();
            }

            match &mut self.volume_areas[0] {
                VolumeArea::BeamVolumeArea(area) => {
                    area.radius += delta*0.2;
                    area.radius = area.radius.abs();

                },
                _ => {}
            }

            match &mut self.volume_areas[1] {
                VolumeArea::SphericalVolumeArea(area) => {
                    area.radius = f32::lerp(
                        0.0,
                        self.target_size,
                        explode_coeff.clamp(0.0, 1.0)
                    );
                    area.radius = area.radius.abs();

                }
                _ => {}
            }

            for wave in self.waves.iter_mut()
            {
                wave.radius = f32::lerp(
                    0.0,
                    self.target_size*1.1,
                    explode_coeff.clamp(0.0, 1.0)
                );
                wave.radius = wave.radius.abs();
            }
            
            match &mut self.volume_areas[2] {
                VolumeArea::SphericalVolumeArea(area) => {
                    area.radius -= delta*0.35;
                    area.radius = area.radius.abs();

                }
                _ => {}
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
                volume_areas: Some(&self.volume_areas),
                waves: Some(&self.waves),
                player: None,
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