use self::collider::{Area, DynamicCollider, StaticCollider};

use super::{
    actor::Actor, world::static_object::StaticObject, world::World
};

use glam::Vec4;

pub mod collider;

struct StaticColliderData {
    postition: Vec4,
    size: Vec4,
    friction: f32,
    bounce_rate: f32,
}

pub struct LevelStaticCollidersData {
     cubes: Vec<StaticColliderData>,
     inf_w_cubes: Vec<StaticColliderData>,
     spheres: Vec<StaticColliderData>,
     shpcubes: Vec<StaticColliderData>,

     neg_cubes: Vec<StaticColliderData>,
     neg_inf_w_cubes: Vec<StaticColliderData>,
     neg_spheres: Vec<StaticColliderData>,
     neg_shpcubes: Vec<StaticColliderData>,
}

impl LevelStaticCollidersData {
    pub fn new(world: &World) -> Self {
        let mut cubes = Vec::new();
        let mut inf_w_cubes = Vec::new();
        let mut spheres = Vec::new();
        let mut shpcubes = Vec::new();

        let mut neg_cubes = Vec::new();
        let mut neg_inf_w_cubes = Vec::new();
        let mut neg_spheres = Vec::new();
        let mut neg_shpcubes = Vec::new();

        for object in world.static_objects.iter() {
            match object {
                StaticObject::Cube(transform, size, is_positive) => {
                    let position = transform.get_position();
                    let size = size.clone();

                    if *is_positive {
                        cubes.push((position, size));
                    } else {
                        neg_cubes.push((position, size));
                    }
                }
                StaticObject::CubeInfW(transform, size, is_positive) => {
                    let position = transform.get_position();
                    let size = size.clone();

                    if *is_positive {
                        inf_w_cubes.push((position, size));
                    } else {
                        neg_inf_w_cubes.push((position, size));
                    }
                }
                StaticObject::SphCube(transform, size, is_positive) => {
                    let position = transform.get_position();
                    let size = size.clone();

                    if *is_positive {
                        shpcubes.push((position, size));
                    } else {
                        neg_shpcubes.push((position, size));
                    }
                }
                StaticObject::Sphere(transform, size, is_positive) => {
                    let position = transform.get_position();
                    let size = size.clone();

                    if *is_positive {
                        spheres.push((position, size));
                    } else {
                        neg_spheres.push((position, size));
                    }
                }
            }
        }

        StaticObjectsData {
            cubes,
            inf_w_cubes,
            spheres,
            shpcubes,
       
            neg_cubes,
            neg_inf_w_cubes,
            neg_spheres,
            neg_shpcubes,
        }
    }
}

pub struct PhysicsSystem {
    static_objects_data: StaticObjectsData,

    frame_static_colliders_buffer: Vec<StaticColliderData>,
    frame_dynamics_colliders_buffer: Vec<DynamicColliderData>
}
pub struct PhysicsState {

}

impl PhysicsState {
    pub fn new() -> Self {
        PhysicsState {  }
    }

    pub fn ray_cast(&mut self, from: Vec4, direction: Vec4, len: f64) -> Option<Hit> {
        None
    }
}

pub struct Hit {
    pub hit_point: Vec4,
    pub hited_players_id: Option<u64>, 
    pub hit_normal: Vec4,
}


impl PhysicsSystem {
    pub fn new(world: &World) -> Self {
        
        let static_objects_data = StaticObjectsData::new(world);

        PhysicsSystem {
            static_objects_data,
            frame_static_colliders_buffer: Vec::with_capacity(20),
            frame_dynamics_colliders_buffer: Vec::with_capacity(1),
            frame_areas_buffer: Vec::with_capacity(20),
        }
    }

    pub fn process_physics(&mut self, world: &mut World, dt: f32) {
        for (_, actor) in world.actors.iter_mut() {

            if let Some(dynamic_collider) = actor.get_dynamic_collider() {
                
                self.frame_dynamics_colliders_buffer.push(dynamic_collider);
            }

            if let Some(static_colliders) = actor.get_static_colliders() {

                for static_collider in static_colliders.iter_mut() {

                    self.frame_static_colliders_buffer.push(static_collider);
                }
            }

            if let Some(areas) = actor.get_areas() {
                
                for area in areas.iter_mut() {

                    self.frame_areas_buffer.push(area);
                }
            }
        }

        self.frame_areas_buffer.clear();
        self.frame_dynamics_colliders_buffer.clear();
        self.frame_static_colliders_buffer.clear();
    }
}