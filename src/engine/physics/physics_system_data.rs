// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2025  Timofei Molokov

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::{
    engine::{
        physics::{
            area::Area, kinematic_collider::KinematicCollider, static_collider::StaticCollider
        }, world::World
    }, transform::Transform
};

use client_server_protocol::Team;
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
    undestroyable_normal: Vec<StaticCollider>,
    undestroyable_stickiness: Vec<StaticCollider>,


    constant_normal_len: usize,
    constant_negative_len: usize,
    constant_stickiness_len: usize,
    constant_neg_stickiness_len: usize,
    constant_undestroyable_normal_len: usize,
    constant_undestroyable_stickiness_len: usize,
}

impl CollidersShapeTypeArrays {
    pub fn new() -> Self {
        let normal = Vec::new();
        let negative = Vec::new();
        let stickiness = Vec::new();
        let neg_stickiness = Vec::new();
        let undestroyable_normal = Vec::new();
        let undestroyable_stickiness = Vec::new();

        let constant_normal_len = 0;
        let constant_negative_len = 0;
        let constant_stickiness_len = 0;
        let constant_neg_stickiness_len = 0;
        let constant_undestroyable_normal_len = 0;
        let constant_undestroyable_stickiness_len = 0;

        CollidersShapeTypeArrays {
            normal,
            negative,
            stickiness,
            neg_stickiness,
            undestroyable_normal,
            undestroyable_stickiness,

            constant_normal_len,
            constant_negative_len,
            constant_stickiness_len,
            constant_neg_stickiness_len,
            constant_undestroyable_normal_len,
            constant_undestroyable_stickiness_len,
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
    pub fn iter_undestroyable_normal(&self) -> std::slice::Iter<'_, StaticCollider>{
        self.undestroyable_normal.iter()
    }

    #[inline]
    pub fn iter_undestroyable_stickiness(&self) -> std::slice::Iter<'_, StaticCollider>{
        self.undestroyable_stickiness.iter()
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
    pub fn get_undestroyable_normal(&self, index: usize) -> &StaticCollider {
        &self.undestroyable_normal[index]
    }

    #[inline]
    pub fn get_undestroyable_stickiness(&self, index: usize) -> &StaticCollider {
        &self.undestroyable_stickiness[index]
    }




    #[inline]
    fn add_constant_static_collider(&mut self, static_collider: StaticCollider) {
        if static_collider.is_positive {
            if static_collider.stickiness{
                if static_collider.undestroyable
                {
                    self.undestroyable_stickiness.push(static_collider);
                    self.constant_undestroyable_stickiness_len += 1;
                }
                else
                {
                    self.stickiness.push(static_collider);
                    self.constant_stickiness_len += 1;
                }
            } else {
                if static_collider.undestroyable
                {
                    self.undestroyable_normal.push(static_collider);
                    self.constant_undestroyable_normal_len += 1;
                }
                else
                {
                    self.normal.push(static_collider);
                    self.constant_normal_len += 1;
                }
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

        if static_collider.is_positive
        {
            if static_collider.stickiness
            {
                if static_collider.undestroyable
                {
                    self.undestroyable_stickiness.push(static_collider);
                }
                else
                {
                    self.stickiness.push(static_collider);
                }
            } else {
                if static_collider.undestroyable
                {
                    self.undestroyable_normal.push(static_collider);
                }
                else
                {
                    self.normal.push(static_collider);
                }
            }
        }
        else
        {
            if static_collider.stickiness
            {
                self.neg_stickiness.push(static_collider);
            }
            else
            {

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
        self.undestroyable_normal.truncate(self.constant_undestroyable_normal_len);
        self.undestroyable_stickiness.truncate(self.constant_undestroyable_stickiness_len);
    }

    fn clear_all_static_colliders(&mut self) {

        self.normal.clear();
        self.negative.clear();
        self.stickiness.clear();
        self.neg_stickiness.clear();
        self.undestroyable_normal.clear();
        self.undestroyable_stickiness.clear();

        self.constant_normal_len = 0;
        self.constant_negative_len = 0;
        self.constant_stickiness_len = 0;
        self.constant_neg_stickiness_len = 0;
        self.constant_undestroyable_normal_len = 0;
        self.constant_undestroyable_stickiness_len = 0;
    }

}

pub struct PhysicsState {
    pub cubes: CollidersShapeTypeArrays,
    pub spheres: CollidersShapeTypeArrays,
    pub sph_cubes: CollidersShapeTypeArrays,
    pub inf_w_cubes: CollidersShapeTypeArrays,

    pub player_forms: Vec<PlayersDollCollider>,

    // pub w_floor: Option<WFloor>,
    // pub w_roof: Option<WRoof>,

    pub stickiness: f32,
}

pub struct Hit {
    pub hit_point: Vec4,
    pub hit_normal: Vec4,
    pub hited_actors_id: Option<u128>,
    pub hited_actors_team: Option<Team>,
}

struct StaticColliderData {
    postition: Vec4,
    size: Vec4,
    friction: f32,
    bounce_rate: f32,
}

impl PhysicsState {
    pub fn new() -> Self {
        let mut cubes = CollidersShapeTypeArrays::new();
        let mut spheres = CollidersShapeTypeArrays::new();
        let mut sph_cubes = CollidersShapeTypeArrays::new();
        let mut inf_w_cubes = CollidersShapeTypeArrays::new();

        
        PhysicsState {
            cubes,
            inf_w_cubes,
            spheres,
            sph_cubes,

            player_forms: Vec::with_capacity(4),

            // w_floor: world.level.w_floor.clone(),
            // w_roof: world.level.w_roof.clone(),

            stickiness: 0.0,
        }
    }

    pub fn update_level_static_info(&mut self, world: &World)
    {
        self.cubes.clear_all_static_colliders();
        self.spheres.clear_all_static_colliders();
        self.sph_cubes.clear_all_static_colliders();
        self.inf_w_cubes.clear_all_static_colliders();

        for static_object in world.level.as_ref().unwrap().static_objects.iter() {

            let collider = static_object.collider.clone();

            match collider.shape_type {
                ShapeType::Cube => {
                    self.cubes.add_constant_static_collider(collider)
                }
                ShapeType::CubeInfW => {
                    self.inf_w_cubes.add_constant_static_collider(collider)
                }
                ShapeType::SphCube => {
                    self.sph_cubes.add_constant_static_collider(collider)
                }
                ShapeType::Sphere => {
                    self.spheres.add_constant_static_collider(collider)
                }
            }
        }

        self.stickiness = world.level.as_ref().unwrap().all_shapes_stickiness_radius;

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
        self.player_forms.push(collider);
    }

    pub fn clear_temporal_colliders(&mut self) {
        self.cubes.clear_temporal_static_colliders();
        self.spheres.clear_temporal_static_colliders();
        self.sph_cubes.clear_temporal_static_colliders();
        self.inf_w_cubes.clear_temporal_static_colliders();

        self.player_forms.clear();
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
