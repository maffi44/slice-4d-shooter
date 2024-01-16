use super::{world::World, static_obj::StaticObject, transform::Position};

use glam::Vec4;

pub mod collisions;

pub struct StaticObjectsData {
     cubes: Vec<(Vec4,Vec4)>,
     inf_w_cubes: Vec<(Vec4,Vec4)>,
     spheres: Vec<(Vec4,Vec4)>,
     shpcubes: Vec<(Vec4,Vec4)>,

     neg_cubes: Vec<(Vec4,Vec4)>,
     neg_inf_w_cubes: Vec<(Vec4,Vec4)>,
     neg_spheres: Vec<(Vec4,Vec4)>,
     neg_shpcubes: Vec<(Vec4,Vec4)>,
}

impl StaticObjectsData {
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
    static_objects_data: StaticObjectsData
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
    pub hited_players_id: Option<u32>, 
    pub hit_normal: Vec4,
}


impl PhysicsSystem {
    pub fn new(world: &World) -> Self {
        
        let static_objects_data = StaticObjectsData::new(world);

        PhysicsSystem {
            static_objects_data
        }
    }

    pub fn process_physics(&mut self, world: &mut World, dt: f32) {
        for player_id in world.spawned_players.iter() {
            if let Some(player) = world.pool_of_players.get_mut(player_id) {
                player.get_mut_collider().physics_tick(dt, &self.static_objects_data);
            }
        }
    }


}