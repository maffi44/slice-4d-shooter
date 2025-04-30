use client_server_protocol::Team;
use glam::Vec4;

use crate::{
    engine::{
        engine_handle::{Command, CommandType}, physics::{
            colliders_container::PhysicalElement,
            physics_system_data::ShapeType,
            static_collider::StaticCollider
        }, render::VisualElement, world::static_object::{
            BeamVolumeArea,
            ColoringArea,
            SphericalVolumeArea,
            StaticObject,
            VolumeArea
        }
    },
    transform::Transform
};

use super::{
    device::shotgun::{
        SHOTGUN_LASER_SHOT_ADD_FORCE_PER_HIT, SHOTGUN_LASER_SHOT_BEAM_RADIUS, SHOTGUN_LASER_SHOT_COLOR, SHOTGUN_LASER_SHOT_DAMAGE, SHOTGUN_LASER_SHOT_EXPLOSION_EXPAND_SPEED, SHOTGUN_LASER_SHOT_EXPLOSION_HOLE_MULT, SHOTGUN_LASER_SHOT_EXPLOSION_MAX_RADIUS, SHOTGUN_LASER_SHOT_HOLE_REDUCTION_SPEED, SHOTGUN_LASER_SHOT_LENGTH, SHOTGUN_LASER_SHOT_MAX_DISTANCE, SHOTGUN_LASER_SHOT_SPEED
    },
    main_player::PlayerMessage,
    Actor,
    ActorID,
    Message,
    MessageType,
    SpecificActorMessage
};



pub struct ShotgunLaserShot
{
    transform: Transform,
    id: Option<ActorID>,
    real_start_position: Vec4,
    visible_start_position: Vec4,
    possible_destination: Vec4,
    real_shot_direction: Vec4,
    visible_shot_direction: Vec4,
    damage_dealer_id: ActorID,
    damage_dealer_team: Team,

    forward_point_of_visible_laser: Vec4,
    real_laser_reached_dist_coef: f32,

    real_reached_distance: f32,
    real_distance: f32,
    visible_distance: f32,

    laser_length: f32,

    static_objects: Vec<StaticObject>,
    coloring_areas: Vec<ColoringArea>,
    volume_areas: Vec<VolumeArea>,

    explosion_max_size_reached: bool,

    is_replicated: bool,

    hited_or_reached_max_distace: bool,
}

impl ShotgunLaserShot
{
    pub fn new(
        real_start_position: Vec4,
        visible_start_position: Vec4,
        possible_destination: Vec4,
        damage_dealer_id: ActorID,
        damage_dealer_team: Team,
        is_replicated: bool,

    ) -> Self
    {
        let transform = Transform::from_position(real_start_position);
        
        let visible_shot_direction = (possible_destination - visible_start_position).normalize();
        if visible_shot_direction.is_nan() {panic!{"catched NAN during creating Shotgun Laser Shot"}}
        
        let real_shot_direction = (possible_destination - real_start_position).normalize();
        
        let hited_or_reached_max_distace = real_shot_direction.is_nan();
        
        let real_distance = (real_start_position - possible_destination).length();
        let visible_distance = (visible_start_position - possible_destination).length();

        let static_objects = Vec::with_capacity(1);
        let coloring_areas = Vec::with_capacity(1);
        let mut volume_areas = Vec::with_capacity(2);

        let laser = BeamVolumeArea {
            translation_pos_1: Vec4::ZERO,
            translation_pos_2: visible_shot_direction*0.01,
            radius: SHOTGUN_LASER_SHOT_BEAM_RADIUS,
            color: SHOTGUN_LASER_SHOT_COLOR,
        };

        volume_areas.push(VolumeArea::BeamVolumeArea(laser));
        
        ShotgunLaserShot
        {
            transform,
            id: None,
            real_start_position,
            visible_start_position,
            possible_destination,
            damage_dealer_id,
            damage_dealer_team,
            real_shot_direction,
            visible_shot_direction,
            forward_point_of_visible_laser: visible_start_position,
            real_laser_reached_dist_coef: 0.0,
            hited_or_reached_max_distace,
            real_distance,
            real_reached_distance: 0.0,
            visible_distance,
            laser_length: SHOTGUN_LASER_SHOT_LENGTH,
            static_objects,
            coloring_areas,
            volume_areas,
            is_replicated,
            explosion_max_size_reached: false,
        }
    }

    fn get_laser_volume_area(&mut self) -> Option<&mut BeamVolumeArea>
    {
        for area in &mut self.volume_areas
        {
            match area
            {
                VolumeArea::BeamVolumeArea(area) => return Some(area),
                _ => {}
            }
        }

        None
    }

    fn get_explosion_volume_area(&mut self) -> Option<&mut SphericalVolumeArea>
    {
        for area in &mut self.volume_areas
        {
            match area
            {
                VolumeArea::SphericalVolumeArea(area) => return Some(area),
                _ => {}
            }
        }

        None
    }

    fn remove_laser_volume_area(&mut self)
    {
        self.volume_areas.retain_mut(|area| {
            if let VolumeArea::BeamVolumeArea(_) = area
            {
                return false;
            }

            true
        });
    }

    fn remove_explode_volume_area(&mut self)
    {
        self.volume_areas.retain_mut(|area| {
            if let VolumeArea::SphericalVolumeArea(_) = area
            {
                return false;
            }

            true
        });
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

    fn get_visual_element(&self) -> Option<VisualElement> {
        Some(VisualElement {
            transform: &self.transform,
            static_objects: Some(&self.static_objects),
            coloring_areas: Some(&self.coloring_areas),
            volume_areas: Some(&self.volume_areas),
            waves: None,
            player: None
        })
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement> {
        Some(PhysicalElement {
            id: self.id.expect("Shotgun Laser Shot have not ActorID"),
            transform: &mut self.transform,
            kinematic_collider: None,
            dynamic_colliders: None,
            static_colliders: None,
            static_objects: Some(&mut self.static_objects),
            area: None
        })
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
        if self.static_objects.len() == 0 &&
            self.volume_areas.len() == 0 &&
            self.coloring_areas.len() == 0
        {
            let my_id = self.id.expect("Shotgun Laser Shot have not ActorID");
            engine_handle.send_command(
                Command {
                    sender: my_id,
                    command_type: CommandType::RemoveActor(my_id)
                }
            );
            return;
        }

        if self.hited_or_reached_max_distace
        {

            if self.explosion_max_size_reached
            {
                let mut del_explosion = false;

                if let Some(explosion) = self.get_explosion_volume_area()
                {
                    explosion.radius -= delta*SHOTGUN_LASER_SHOT_EXPLOSION_EXPAND_SPEED;

                    if explosion.radius < 0.01
                    {
                        del_explosion = true;
                    }
                }

                if del_explosion {self.remove_explode_volume_area();}


                if self.static_objects.len() > 0
                {
                    
                    self.static_objects[0].collider.size.x -=
                        delta*SHOTGUN_LASER_SHOT_HOLE_REDUCTION_SPEED;

                    if self.static_objects[0].collider.size.x <= 0.01
                    {
                        self.static_objects.clear();
                    }
                }
                if self.coloring_areas.len() > 0
                {
                    self.coloring_areas[0].radius -=
                        delta*SHOTGUN_LASER_SHOT_HOLE_REDUCTION_SPEED;
                    
                    if self.coloring_areas[0].radius <= 0.01
                    {
                        self.coloring_areas.clear();
                    }
                }
            }
            else
            {
                if let Some(explosion) = self.get_explosion_volume_area()
                {
                    explosion.radius += delta*SHOTGUN_LASER_SHOT_EXPLOSION_EXPAND_SPEED;

                    if explosion.radius >= SHOTGUN_LASER_SHOT_EXPLOSION_MAX_RADIUS
                    {
                        self.explosion_max_size_reached = true;
                    }
                }

                if self.static_objects.len() > 0
                {
                    self.static_objects[0].collider.size.x +=
                        delta*SHOTGUN_LASER_SHOT_EXPLOSION_EXPAND_SPEED*SHOTGUN_LASER_SHOT_EXPLOSION_HOLE_MULT;
                }

                if self.coloring_areas.len() > 0
                {
                    self.coloring_areas[0].radius +=
                        delta*SHOTGUN_LASER_SHOT_EXPLOSION_EXPAND_SPEED*SHOTGUN_LASER_SHOT_EXPLOSION_HOLE_MULT;
                }
            }

            if self.laser_length > 0.0
            {
                self.laser_length -= SHOTGUN_LASER_SHOT_SPEED*delta;

                if self.laser_length <= 0.0
                {
                    self.remove_laser_volume_area();
                }
                else
                {
                    let backward_point_of_visible_laser = {
                        self.forward_point_of_visible_laser -
                        self.visible_shot_direction *
                        (
                            self.visible_distance *
                            self.real_laser_reached_dist_coef.min(self.laser_length/self.visible_distance)
                        )
                    };
    
                    let real_laser_forward_position = self.transform.get_position();
    
                    let laser = self
                        .get_laser_volume_area()
                        .expect("Shotgun Laser Shot haven't beam volume area");
    
                    laser.translation_pos_2 = backward_point_of_visible_laser - real_laser_forward_position;
                }
            }
        }
        else
        {
            if let Some(hit) = physic_system.ray_cast(
                self.transform.get_position(),
                self.real_shot_direction,
                SHOTGUN_LASER_SHOT_SPEED * delta,
                Some(self.damage_dealer_id)
            )
            {
                if let Some(hited_id) = hit.hited_actors_id
                {
                    if !self.is_replicated
                    {
                        engine_handle.send_direct_message(
                            hited_id,
                            Message {
                                from: self.id.expect("Shotgun Laser Shot havn't ActorID "),
                                remote_sender: false,
                                message: MessageType::SpecificActorMessage(
                                    SpecificActorMessage::PlayerMessage(
                                        PlayerMessage::DealDamageAndAddForce(
                                            SHOTGUN_LASER_SHOT_DAMAGE,
                                            self.real_shot_direction*SHOTGUN_LASER_SHOT_ADD_FORCE_PER_HIT,
                                            hit.hit_point,
                                            self.damage_dealer_team,
                                            self.damage_dealer_id
                                        )
                                    )
                                )
                            }
                        );
                    }
                }

                self.transform.set_position(hit.hit_point);

                let explode = SphericalVolumeArea {
                    translation: Vec4::ZERO,
                    radius: 0.001,
                    color: SHOTGUN_LASER_SHOT_COLOR
                };

                self.volume_areas.push(VolumeArea::SphericalVolumeArea(explode));

                let hole = StaticObject {
                    collider: StaticCollider {
                        position: Vec4::ZERO,
                        size: Vec4::new(0.001, 0.0, 0.0, 0.0),
                        is_positive: false,
                        roundness: 0.0,
                        stickiness: false,
                        friction: 0.0,
                        bounce_rate: 0.0,
                        shape_type: ShapeType::Sphere,
                        actors_id: None,
                        
                    },
                    material_index: 0
                };

                self.static_objects.push(hole);

                let coloring_area = ColoringArea {
                    translation: Vec4::ZERO,
                    radius: 0.021,
                    color: SHOTGUN_LASER_SHOT_COLOR,
                };

                self.coloring_areas.push(coloring_area);

                self.hited_or_reached_max_distace = true;
            }
            else
            {
                let frame_offset = SHOTGUN_LASER_SHOT_SPEED*delta; 
                self.transform.set_position(
                    self.transform.get_position() +
                    (self.real_shot_direction*frame_offset)
                );

                self.real_reached_distance += frame_offset;
                self.real_laser_reached_dist_coef = self.real_reached_distance / self.real_distance;

                self.forward_point_of_visible_laser = {
                    self.visible_start_position +
                    self.visible_shot_direction * (self.visible_distance * self.real_laser_reached_dist_coef)
                };

                let backward_point_of_visible_laser = {
                    self.forward_point_of_visible_laser -
                    self.visible_shot_direction *
                    (
                        self.visible_distance *
                        self.real_laser_reached_dist_coef.min(self.laser_length/self.visible_distance)
                    )
                };

                let real_laser_forward_position = self.transform.get_position();
                let forward_point_of_visible_laser = self.forward_point_of_visible_laser;

                let laser = self
                    .get_laser_volume_area()
                    .expect("Shotgun Laser Shot haven't beam volume area");

                laser.translation_pos_1 = forward_point_of_visible_laser - real_laser_forward_position;
                laser.translation_pos_2 = backward_point_of_visible_laser - real_laser_forward_position;


                if self.real_reached_distance >= SHOTGUN_LASER_SHOT_MAX_DISTANCE
                {
                    self.hited_or_reached_max_distace = true;
                }
            }
        }
    }
}