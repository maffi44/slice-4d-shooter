use crate::{
    transform::Transform,
    engine::{
        world::World,
        physics::{
            kinematic_collider::KinematicCollider,
            // dynamic_collider::DynamicCollider,
            static_collider::StaticCollider,
            area::Area,
        }
    },
};

use glam::Vec4;

use super::dynamic_collider::PlayersDollCollider;



#[derive(Debug, Clone)]
pub enum ShapeType {
    Cube,
    CubeInfW,
    Sphere,
    SphCube,
}

pub struct CollidersShapeTypeArrays {
    normal: Vec<StaticCollider>,
    negative: Vec<StaticCollider>,
    stickiness: Vec<StaticCollider>,
    neg_stickiness: Vec<StaticCollider>,


    constant_normal_len: usize,
    constant_negative_len: usize,
    constant_stickiness_len: usize,
    constant_neg_stickiness_len: usize,
}

impl CollidersShapeTypeArrays {
    pub fn new() -> Self {
        let normal = Vec::new();
        let negative = Vec::new();
        let stickiness = Vec::new();
        let neg_stickiness = Vec::new();

        let constant_normal_len = 0;
        let constant_negative_len = 0;
        let constant_stickiness_len = 0;
        let constant_neg_stickiness_len = 0;

        CollidersShapeTypeArrays {
            normal,
            negative,
            stickiness,
            neg_stickiness,

            constant_normal_len,
            constant_negative_len,
            constant_stickiness_len,
            constant_neg_stickiness_len,
        }
    }



    #[inline]
    pub fn iter_normal(&self) -> std::slice::Iter<'_, StaticCollider>{
        self.normal.iter()
    }

    #[inline]
    pub fn iter_negative(&self) -> std::slice::Iter<'_, StaticCollider>{
        self.negative.iter()
    }

    #[inline]
    pub fn iter_stickiness(&self) -> std::slice::Iter<'_, StaticCollider>{
        self.stickiness.iter()
    }

    #[inline]
    pub fn iter_neg_stickiness(&self) -> std::slice::Iter<'_, StaticCollider>{
        self.neg_stickiness.iter()
    }



    #[inline]
    pub fn get_normal(&self, index: usize) -> &StaticCollider {
        &self.normal[index]
    }

    #[inline]
    pub fn get_stickiness(&self, index: usize) -> &StaticCollider {
        &self.stickiness[index]
    }

    #[inline]
    pub fn get_negative(&self, index: usize) -> &StaticCollider {
        &self.negative[index]
    }

    #[inline]
    pub fn get_neg_stickiness(&self, index: usize) -> &StaticCollider {
        &self.neg_stickiness[index]
    }



    #[inline]
    fn add_constant_static_collider(&mut self, static_collider: StaticCollider) {

        if static_collider.is_positive {
            if static_collider.stickiness{

                self.stickiness.push(static_collider);
                self.constant_stickiness_len += 1;
            } else {

                self.normal.push(static_collider);
                self.constant_normal_len += 1;
            }
        } else {
            if static_collider.stickiness{

                self.neg_stickiness.push(static_collider);
                self.constant_neg_stickiness_len += 1;
            } else {

                self.negative.push(static_collider);
                self.constant_negative_len += 1;
            }
        }

    }



    #[inline]
    fn add_temporal_static_collider(&mut self, static_collider: StaticCollider) {
        if static_collider.is_positive {
            if static_collider.stickiness{

                self.stickiness.push(static_collider);
            } else {

                self.normal.push(static_collider);
            }
        } else {
            if static_collider.stickiness{

                self.neg_stickiness.push(static_collider);
            } else {

                self.negative.push(static_collider);
            }
        }
    }


    #[inline]
    fn clear_temporal_static_colliders(&mut self) {

        self.normal.truncate(self.constant_normal_len);
        self.negative.truncate(self.constant_negative_len);
        self.stickiness.truncate(self.constant_stickiness_len);
        self.neg_stickiness.truncate(self.constant_neg_stickiness_len);
    }

}

pub struct PhysicsState {
    pub cubes: CollidersShapeTypeArrays,
    pub spheres: CollidersShapeTypeArrays,
    pub sph_cubes: CollidersShapeTypeArrays,
    pub inf_w_cubes: CollidersShapeTypeArrays,

    pub dyn_spheres: Vec<PlayersDollCollider>,

    pub stickiness: f32,
}

pub struct Hit {
    pub hit_point: Vec4,
    pub hit_normal: Vec4,
    pub hited_actors_id: Option<u64>, 
}

struct StaticColliderData {
    postition: Vec4,
    size: Vec4,
    friction: f32,
    bounce_rate: f32,
}

impl PhysicsState {
    pub fn new(world: &World) -> Self {
        let mut cubes = CollidersShapeTypeArrays::new();
        let mut spheres = CollidersShapeTypeArrays::new();
        let mut sph_cubes = CollidersShapeTypeArrays::new();
        let mut inf_w_cubes = CollidersShapeTypeArrays::new();

        for static_object in world.level.static_objects.iter() {

            let collider = static_object.collider.clone();

            match collider.shape_type {
                ShapeType::Cube => {
                    cubes.add_constant_static_collider(collider)
                }
                ShapeType::CubeInfW => {
                    inf_w_cubes.add_constant_static_collider(collider)
                }
                ShapeType::SphCube => {
                    sph_cubes.add_constant_static_collider(collider)
                }
                ShapeType::Sphere => {
                    spheres.add_constant_static_collider(collider)
                }
            }
        }

        PhysicsState {
            cubes,
            inf_w_cubes,
            spheres,
            sph_cubes,

            dyn_spheres: Vec::with_capacity(4),

            stickiness: world.level.all_shapes_stickiness_radius
        }
    }

    pub fn add_temporal_static_collider(&mut self, collider: StaticCollider) {
        match collider.shape_type {
            ShapeType::Cube => {
                self.cubes.add_temporal_static_collider(collider)
            }
            ShapeType::CubeInfW => {
                self.inf_w_cubes.add_temporal_static_collider(collider)
            }
            ShapeType::SphCube => {
                self.sph_cubes.add_temporal_static_collider(collider)
            }
            ShapeType::Sphere => {
                self.spheres.add_temporal_static_collider(collider)
            }
        }
    }

    pub fn add_temporal_dynamic_collider(&mut self, collider: PlayersDollCollider) {
        self.dyn_spheres.push(collider);
    }

    pub fn clear_temporal_colliders(&mut self) {
        self.cubes.clear_temporal_static_colliders();
        self.spheres.clear_temporal_static_colliders();
        self.sph_cubes.clear_temporal_static_colliders();
        self.inf_w_cubes.clear_temporal_static_colliders();

        self.dyn_spheres.clear();
    }
}


pub struct FrameCollidersBuffers {
    pub dynamic_colliders: Vec<&'static mut PlayersDollCollider>,
    pub kinematic_colliders: Vec<(&'static mut Transform, &'static mut KinematicCollider)>,
    pub areas: Vec<&'static mut Area>,
}

impl FrameCollidersBuffers {
    pub fn new() -> Self {
        let dynamic_colliders = Vec::<&mut PlayersDollCollider>::new();
        let kinematic_colliders =
            Vec::<(&'static mut Transform, &'static mut KinematicCollider)>::new();
        
        let areas =
            Vec::<&mut Area>::new();
        

        FrameCollidersBuffers {
            dynamic_colliders,
            kinematic_colliders,
            areas,
        }
    }
}
