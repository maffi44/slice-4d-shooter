use crate::{
    actor::{
        main_player::{MainPlayer, PlayerProjection, PlayerScreenEffects},
        Actor,
        ActorWrapper
    },
    engine::{
        physics::physics_system_data::ShapeType,
        render::{camera::Camera, render_data::{
            Shape,
            ShapesArrays,
            ShapesArraysMetadata,
            SphericalArea,
            SphericalAreasMetadata
        }},
        time::TimeSystem,
        world::{
            static_object::{VisualWave, VolumeArea},
            World
        }
    }, transform::Transform
};

use std::f32::consts::PI;

use client_server_protocol::Team;
use glam::{Mat4, Vec4};
use winit::window::Window;

use super::{static_render_data::StaticRenderData, BeamArea, BoundingBox, PlayerForm};



pub struct DynamicRenderData {
    pub dynamic_shapes_data: ShapesArrays,
    pub spherical_areas_data: Box<[SphericalArea; 256]>,
    pub beam_areas_data: Box<[BeamArea; 64]>,
    pub player_forms_data: Box<[PlayerForm; 16]>,
    pub other_dynamic_data: OtherDynamicData,

    // frame memory buffers
    frame_cubes_buffer: SpecificShapeBuffers,
    frame_spheres_buffer: SpecificShapeBuffers,
    frame_sph_cubes_buffer: SpecificShapeBuffers,
    frame_inf_w_cubes_buffer: SpecificShapeBuffers,

    frame_coloring_areas_buffer: Vec<SphericalArea>,
    frame_spherical_volume_areas_buffer: Vec<SphericalArea>,
    frame_waves_buffer: Vec<SphericalArea>,
    frame_beam_volume_areas_buffer: Vec<BeamArea>,
    frame_player_forms_buffer: Vec<PlayerForm>,
}

impl DynamicRenderData {
    pub fn new() -> Self {
        DynamicRenderData {
            dynamic_shapes_data: ShapesArrays::default(),
            spherical_areas_data: {Box::new([SphericalArea::default(); 256])},
            beam_areas_data: {Box::new([BeamArea::default(); 64])},
            player_forms_data: {Box::new([PlayerForm::default(); 16])},
            other_dynamic_data: OtherDynamicData::default(),

            frame_cubes_buffer: SpecificShapeBuffers::default(),
            frame_spheres_buffer: SpecificShapeBuffers::default(),
            frame_sph_cubes_buffer: SpecificShapeBuffers::default(),
            frame_inf_w_cubes_buffer: SpecificShapeBuffers::default(),

            frame_coloring_areas_buffer: Vec::new(),
            frame_spherical_volume_areas_buffer: Vec::new(),
            frame_beam_volume_areas_buffer: Vec::new(),
            frame_player_forms_buffer: Vec::new(),
            frame_waves_buffer: Vec::new(),
        }
    }

    fn get_data_from_actors_visual_elements(
        &mut self,
        world: &World,
        static_bounding_box: &BoundingBox
    ) -> BoundingBox {
        
        let mut frame_bounding_box = static_bounding_box.clone();
        
        for (_, actor) in world.actors.iter() {

            if let Some(visual_element) = actor.get_visual_element() {

                let transform = visual_element.transform;

                if let Some(static_objects) = visual_element.static_objects {
                    for static_object in static_objects {
                        
                        let position = static_object.collider.position + transform.get_position();
                        let size = static_object.collider.size * transform.get_scale();
                        let material_index = static_object.material_index;
                        let roundness = static_object.collider.roundness;
                        
                        let shape = Shape {
                            pos: position.to_array(),
                            size: size.to_array(),
                            material: material_index,
                            empty_bytes: [0,0],
                            roundness,
                        };
                        frame_bounding_box.expand_by_shape(&shape);
    
                        let is_positive = static_object.collider.is_positive;
                        let is_stickiness = static_object.collider.stickiness;
                        
                        match static_object.collider.shape_type {
    
                            ShapeType::Cube => {
                                if is_positive {
                                    if is_stickiness {
                                        self.frame_cubes_buffer.stickiness.push(shape);
                                    } else {
                                        self.frame_cubes_buffer.normal.push(shape);
                                    }
                                } else {
                                    if is_stickiness {
                                        self.frame_cubes_buffer.neg_stickiness.push(shape);
                                    } else {
                                        self.frame_cubes_buffer.negative.push(shape);
                                    }
                                }
                            },
                            ShapeType::Sphere => {
                                if is_positive {
                                    if is_stickiness {
                                        self.frame_spheres_buffer.stickiness.push(shape);
                                    } else {
                                        self.frame_spheres_buffer.normal.push(shape);
                                    }
                                } else {
                                    if is_stickiness {
                                        self.frame_spheres_buffer.neg_stickiness.push(shape);
                                    } else {
                                        self.frame_spheres_buffer.negative.push(shape);
                                    }
                                }
                                
                            },
                            ShapeType::SphCube => {
                                if is_positive {
                                    if is_stickiness {
                                        self.frame_sph_cubes_buffer.stickiness.push(shape);
                                    } else {
                                        self.frame_sph_cubes_buffer.normal.push(shape);
                                    }
                                } else {
                                    if is_stickiness {
                                        self.frame_sph_cubes_buffer.neg_stickiness.push(shape);
                                    } else {
                                        self.frame_sph_cubes_buffer.negative.push(shape);
                                    }
                                }
                                
                            },
                            ShapeType::CubeInfW => {
                                if is_positive {
                                    if is_stickiness {
                                        self.frame_inf_w_cubes_buffer.stickiness.push(shape);
                                    } else {
                                        self.frame_inf_w_cubes_buffer.normal.push(shape);
                                    }
                                } else {
                                    if is_stickiness {
                                        self.frame_inf_w_cubes_buffer.neg_stickiness.push(shape);
                                    } else {
                                        self.frame_inf_w_cubes_buffer.negative.push(shape);
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(volume_areas) = visual_element.volume_areas {
                    
                    for volume_area in volume_areas {

                        match volume_area {
                            VolumeArea::SphericalVolumeArea(spherical_area) => {
                                let area = SphericalArea {
                                    pos: (spherical_area.translation + transform.get_position()).to_array(),
                                    radius: spherical_area.radius,
                                    color: spherical_area.color.to_array(),
                                };
        
                                self.frame_spherical_volume_areas_buffer.push(area)
                            },

                            VolumeArea::BeamVolumeArea(beam_area) => {
                                let area = BeamArea {
                                    pos1: (beam_area.translation_pos_1 + transform.get_position()).to_array(),
                                    pos2: (beam_area.translation_pos_2 + transform.get_position()).to_array(),
                                    radius: beam_area.radius,
                                    color: beam_area.color.to_array(),
                                };
        
                                self.frame_beam_volume_areas_buffer.push(area);
                            }
                        }

                    }
                }

                if let Some(coloring_areas) = visual_element.coloring_areas {
                    
                    for coloring_area in coloring_areas {
                        let area = SphericalArea {
                            pos: (coloring_area.translation + transform.get_position()).to_array(),
                            radius: coloring_area.radius,
                            color: coloring_area.color.to_array(),
                        };

                        self.frame_coloring_areas_buffer.push(area)
                    }
                }

                if let Some(visual_waves) = visual_element.waves
                {
                    for wave in visual_waves
                    {
                        let area = SphericalArea {
                            pos: (wave.translation + transform.get_position()).to_array(),
                            radius: wave.radius,
                            color: wave.color.to_array(),
                        };

                        self.frame_waves_buffer.push(area);
                    }
                }

                if let Some((player_sphere, team)) = visual_element.player {

                    let player_form = match team {
                        Team::Red =>
                        {
                            PlayerForm {
                                pos: (player_sphere.position + transform.get_position()).to_array(),
                                is_red: [1;4],
                                color: [1.0, 0.0, 0.0],
                                radius: player_sphere.radius,
                                rotation: actor.get_transform().get_rotation().to_cols_array(),
                                weapon_offset: player_sphere.weapon_offset.to_array()
                            }
                        }
                        Team::Blue =>
                        {
                            PlayerForm {
                                pos: (player_sphere.position + transform.get_position()).to_array(),
                                is_red: [0;4],
                                color: [1.0, 0.0, 0.0],
                                radius: player_sphere.radius,
                                rotation: actor.get_transform().get_rotation().to_cols_array(),
                                weapon_offset: player_sphere.weapon_offset.to_array()
                            }
                        }
                    };
                    
                    frame_bounding_box.expand_by_player_form(&player_form);

                    self.frame_player_forms_buffer.push(player_form);
                }
            }
        };

        frame_bounding_box
    }

    fn clear_all_frame_buffers(&mut self) {
        self.frame_cubes_buffer.clear_buffers();
        self.frame_spheres_buffer.clear_buffers();
        self.frame_sph_cubes_buffer.clear_buffers();
        self.frame_inf_w_cubes_buffer.clear_buffers();

        self.frame_coloring_areas_buffer.clear();
        self.frame_spherical_volume_areas_buffer.clear();
        self.frame_beam_volume_areas_buffer.clear();
        self.frame_player_forms_buffer.clear();
        self.frame_waves_buffer.clear();
    }

    pub fn update_dynamic_shapes_buffers_and_get_metadata(
        &mut self,
        sd: &StaticRenderData,

        camera: &Camera,
        clip_planes: (Vec4, Vec4, Vec4, Vec4),
        stickiness_value: f32,

    ) -> ShapesArraysMetadata
    {
        let mut cubes_start = 0u32;
        let mut cubes_amount = 0u32;

        let mut spheres_start = 0u32;
        let mut spheres_amount = 0u32;

        let mut inf_cubes_start = 0u32;
        let mut inf_cubes_amount = 0u32;

        let mut sph_cubes_start = 0u32;
        let mut sph_cubes_amount = 0u32;


        let mut s_cubes_start = 0u32;
        let mut s_cubes_amount = 0u32;

        let mut s_spheres_start = 0u32;
        let mut s_spheres_amount = 0u32;

        let mut s_inf_cubes_start = 0u32;
        let mut s_inf_cubes_amount = 0u32;

        let mut s_sph_cubes_start = 0u32;
        let mut s_sph_cubes_amount = 0u32;


        let mut neg_cubes_start = 0u32;
        let mut neg_cubes_amount = 0u32;

        let mut neg_spheres_start = 0u32;
        let mut neg_spheres_amount = 0u32;

        let mut neg_inf_cubes_start = 0u32;
        let mut neg_inf_cubes_amount = 0u32;

        let mut neg_sph_cubes_start = 0u32;
        let mut neg_sph_cubes_amount = 0u32;


        let mut s_neg_cubes_start = 0u32;
        let mut s_neg_cubes_amount = 0u32;

        let mut s_neg_spheres_start = 0u32;
        let mut s_neg_spheres_amount = 0u32;

        let mut s_neg_inf_cubes_start = 0u32;
        let mut s_neg_inf_cubes_amount = 0u32;

        let mut s_neg_sph_cubes_start = 0u32;
        let mut s_neg_sph_cubes_amount = 0u32;



        // packing normal shapes
        let mut index = 0;
        cubes_start = 0u32;

        for shape in &sd.cubes
        {
            if check_if_player_see_cube(
                camera,
                Vec4::from_array(shape.pos),
                Vec4::from_array(shape.size) + shape.roundness + stickiness_value,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.normal[index] = *shape;
                index += 1;
            }
        }

        while let Some(shape) = self.frame_cubes_buffer.normal.pop()
        {
            if check_if_player_see_cube(
                camera,
                Vec4::from_array(shape.pos),
                Vec4::from_array(shape.size) + shape.roundness + stickiness_value,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.normal[index] = shape;
                index += 1;
            }
        }

        cubes_amount = index as u32;


        spheres_start = index as u32;

        for shape in &sd.spheres {
            if check_if_player_see_sphere(
                camera,
                Vec4::from_array(shape.pos),
                shape.size[0] + shape.roundness + stickiness_value*PI,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.normal[index] = *shape;
                index += 1;
            }
        }

        while let Some(shape) = self.frame_spheres_buffer.normal.pop() {
            if check_if_player_see_sphere(
                camera,
                Vec4::from_array(shape.pos),
                shape.size[0] + shape.roundness + stickiness_value*PI,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.normal[index] = shape;
                index += 1;
            }
        }

        spheres_amount = index as u32 - spheres_start;


        sph_cubes_start = index as u32;

        for shape in &sd.sph_cubes {
            if check_if_player_see_cube(
                camera,
                Vec4::from_array(shape.pos),
                Vec4::new(
                    (shape.size[1].min(shape.size[2])).min(shape.size[3]),    
                    (shape.size[0].min(shape.size[2])).min(shape.size[3]),    
                    (shape.size[1].min(shape.size[0])).min(shape.size[3]),
                    shape.size[3]
                ) + shape.roundness + stickiness_value,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.normal[index] = *shape;
                index += 1;
            }
        }

        while let Some(shape) = self.frame_sph_cubes_buffer.normal.pop() {
            if check_if_player_see_cube(
                camera,
                Vec4::from_array(shape.pos),
                Vec4::new(
                    (shape.size[1].min(shape.size[2])).min(shape.size[3]),    
                    (shape.size[0].min(shape.size[2])).min(shape.size[3]),    
                    (shape.size[1].min(shape.size[0])).min(shape.size[3]),
                    shape.size[3]
                ) + shape.roundness + stickiness_value,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.normal[index] = shape;
                index += 1;
            }
        }

        sph_cubes_amount = index as u32 - sph_cubes_start;


        inf_cubes_start = index as u32;

        // for shape in &sd.inf_w_cubes {
        //     self.dynamic_shapes_data.normal[index] = shape.clone();
        //     index += 1;
        // }

        // while let Some(shape) = self.frame_inf_w_cubes_buffer.normal.pop() {
        //     self.dynamic_shapes_data.normal[index] = shape;
        //     index += 1;
        // }

        inf_cubes_amount = index as u32 - inf_cubes_start;


        // packing stickiness shapes
        let mut index = 0;
        s_cubes_start = 0u32;

        for shape in &sd.s_cubes {
            if check_if_player_see_cube(
                camera,
                Vec4::from_array(shape.pos),
                Vec4::from_array(shape.size) + shape.roundness + stickiness_value,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.stickiness[index] = *shape;
                index += 1;
            }
        }

        while let Some(shape) = self.frame_cubes_buffer.stickiness.pop() {
            if check_if_player_see_cube(
                camera,
                Vec4::from_array(shape.pos),
                Vec4::from_array(shape.size) + shape.roundness + stickiness_value,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.stickiness[index] = shape;
                index += 1;
            }
        }

        s_cubes_amount = index as u32;


        s_spheres_start = index as u32;

        for shape in &sd.s_spheres {
            if check_if_player_see_sphere(
                camera,
                Vec4::from_array(shape.pos),
                shape.size[0] + shape.roundness + stickiness_value*PI,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.stickiness[index] = *shape;
                index += 1;
            }
        }

        while let Some(shape) = self.frame_spheres_buffer.stickiness.pop() {
            if check_if_player_see_sphere(
                camera,
                Vec4::from_array(shape.pos),
                shape.size[0] + shape.roundness + stickiness_value*PI,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.stickiness[index] = shape;
                index += 1;
            }
        }

        s_spheres_amount = index as u32 - s_spheres_start;


        s_sph_cubes_start = index as u32;

        for shape in &sd.s_sph_cubes {
            if check_if_player_see_cube(
                camera,
                Vec4::from_array(shape.pos),
                Vec4::new(
                    (shape.size[1].min(shape.size[2])).min(shape.size[3]),    
                    (shape.size[0].min(shape.size[2])).min(shape.size[3]),    
                    (shape.size[1].min(shape.size[0])).min(shape.size[3]),
                    shape.size[3]
                ) + shape.roundness + stickiness_value,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.stickiness[index] = *shape;
                index += 1;
            }
        }

        while let Some(shape) = self.frame_sph_cubes_buffer.stickiness.pop() {
            if check_if_player_see_cube(
                camera,
                Vec4::from_array(shape.pos),
                Vec4::new(
                    (shape.size[1].min(shape.size[2])).min(shape.size[3]),    
                    (shape.size[0].min(shape.size[2])).min(shape.size[3]),    
                    (shape.size[1].min(shape.size[0])).min(shape.size[3]),
                    shape.size[3]
                ) + shape.roundness + stickiness_value,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.stickiness[index] = shape;
                index += 1;
            }
        }

        s_sph_cubes_amount = index as u32 - s_sph_cubes_start;



        s_inf_cubes_start = index as u32;

        // for shape in &sd.s_inf_w_cubes {
        //     self.dynamic_shapes_data.stickiness[index] = shape.clone();
        //     index += 1;
        // }

        // while let Some(shape) = self.frame_inf_w_cubes_buffer.stickiness.pop() {
        //     self.dynamic_shapes_data.stickiness[index] = shape;
        //     index += 1;
        // }

        s_inf_cubes_amount = index as u32 - s_inf_cubes_start;



        // packing negative shapes
        let mut index = 0;
        neg_cubes_start = 0u32;

        for shape in &sd.neg_cubes {
            if check_if_player_see_cube(
                camera,
                Vec4::from_array(shape.pos),
                Vec4::from_array(shape.size) + shape.roundness + stickiness_value,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.negative[index] = *shape;
                index += 1;
            }
        }

        while let Some(shape) = self.frame_cubes_buffer.negative.pop() {
            if check_if_player_see_cube(
                camera,
                Vec4::from_array(shape.pos),
                Vec4::from_array(shape.size) + shape.roundness + stickiness_value,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.negative[index] = shape;
                index += 1;
            }
        }

        neg_cubes_amount = index as u32;


        neg_spheres_start = index as u32;

        for shape in &sd.neg_spheres {
            if check_if_player_see_sphere(
                camera,
                Vec4::from_array(shape.pos),
                shape.size[0] + shape.roundness + stickiness_value*PI,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.negative[index] = *shape;
                index += 1;
            }
        }

        while let Some(shape) = self.frame_spheres_buffer.negative.pop() {
            if check_if_player_see_sphere(
                camera,
                Vec4::from_array(shape.pos),
                shape.size[0] + shape.roundness + stickiness_value*PI,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.negative[index] = shape;
                index += 1;
            }
        }

        neg_spheres_amount = index as u32 - neg_spheres_start;


        neg_sph_cubes_start = index as u32;

        for shape in &sd.neg_sph_cubes {
            if check_if_player_see_cube(
                camera,
                Vec4::from_array(shape.pos),
                Vec4::new(
                    (shape.size[1].min(shape.size[2])).min(shape.size[3]),    
                    (shape.size[0].min(shape.size[2])).min(shape.size[3]),    
                    (shape.size[1].min(shape.size[0])).min(shape.size[3]),
                    shape.size[3]
                ) + shape.roundness + stickiness_value,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.negative[index] = *shape;
                index += 1;
            }
        }

        while let Some(shape) = self.frame_sph_cubes_buffer.negative.pop() {
            if check_if_player_see_cube(
                camera,
                Vec4::from_array(shape.pos),
                Vec4::new(
                    (shape.size[1].min(shape.size[2])).min(shape.size[3]),    
                    (shape.size[0].min(shape.size[2])).min(shape.size[3]),    
                    (shape.size[1].min(shape.size[0])).min(shape.size[3]),
                    shape.size[3]
                ) + shape.roundness + stickiness_value,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.negative[index] = shape;
                index += 1;
            }
        }

        neg_sph_cubes_amount = index as u32 - neg_sph_cubes_start;



        neg_inf_cubes_start = index as u32;

        // for shape in &sd.neg_inf_w_cubes {
        //     self.dynamic_shapes_data.negative[index] = shape.clone();
        //     index += 1;
        // }

        // while let Some(shape) = self.frame_inf_w_cubes_buffer.negative.pop() {
        //     self.dynamic_shapes_data.negative[index] = shape;
        //     index += 1;
        // }

        neg_inf_cubes_amount = index as u32 - neg_inf_cubes_start;



        // packing negative and stickiness shapes
        let mut index = 0;
        s_neg_cubes_start = 0u32;

        for shape in &sd.s_neg_cubes {
            if check_if_player_see_cube(
                camera,
                Vec4::from_array(shape.pos),
                Vec4::from_array(shape.size) + shape.roundness + stickiness_value,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.neg_stickiness[index] = *shape;
                index += 1;
            }
        }

        while let Some(shape) = self.frame_cubes_buffer.neg_stickiness.pop() {
            if check_if_player_see_cube(
                camera,
                Vec4::from_array(shape.pos),
                Vec4::from_array(shape.size) + shape.roundness + stickiness_value,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.neg_stickiness[index] = shape;
                index += 1;
            }
        }

        s_neg_cubes_amount = index as u32;


        s_neg_spheres_start = index as u32;

        for shape in &sd.s_neg_spheres {
            if check_if_player_see_sphere(
                camera,
                Vec4::from_array(shape.pos),
                shape.size[0] + shape.roundness + stickiness_value*PI,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.neg_stickiness[index] = *shape;
                index += 1;
            }
        }

        while let Some(shape) = self.frame_spheres_buffer.neg_stickiness.pop() {
            if check_if_player_see_sphere(
                camera,
                Vec4::from_array(shape.pos),
                shape.size[0] + shape.roundness + stickiness_value*PI,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.neg_stickiness[index] = shape;
                index += 1;
            }
        }

        s_neg_spheres_amount = index as u32 - s_neg_spheres_start;


        s_neg_sph_cubes_start = index as u32;

        for shape in &sd.s_neg_sph_cubes {
            if check_if_player_see_cube(
                camera,
                Vec4::from_array(shape.pos),
                Vec4::new(
                    (shape.size[1].min(shape.size[2])).min(shape.size[3]),    
                    (shape.size[0].min(shape.size[2])).min(shape.size[3]),    
                    (shape.size[1].min(shape.size[0])).min(shape.size[3]),
                    shape.size[3]
                ) + shape.roundness + stickiness_value,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.neg_stickiness[index] = *shape;
                index += 1;
            }
        }

        while let Some(shape) = self.frame_sph_cubes_buffer.neg_stickiness.pop() {
            if check_if_player_see_cube(
                camera,
                Vec4::from_array(shape.pos),
                Vec4::new(
                    (shape.size[1].min(shape.size[2])).min(shape.size[3]),    
                    (shape.size[0].min(shape.size[2])).min(shape.size[3]),    
                    (shape.size[1].min(shape.size[0])).min(shape.size[3]),
                    shape.size[3]
                ) + shape.roundness + stickiness_value,
                clip_planes,
            )
            {
                self.dynamic_shapes_data.neg_stickiness[index] = shape;
                index += 1;
            }
        }

        s_neg_sph_cubes_amount = index as u32 - s_neg_sph_cubes_start;



        s_neg_inf_cubes_start = index as u32;

        // for shape in &sd.s_neg_inf_w_cubes {
        //     self.dynamic_shapes_data.neg_stickiness[index] = shape.clone();
        //     index += 1;
        // }

        // while let Some(shape) = self.frame_inf_w_cubes_buffer.neg_stickiness.pop() {
        //     self.dynamic_shapes_data.neg_stickiness[index] = shape;
        //     index += 1;
        // }

        s_neg_inf_cubes_amount = index as u32 - s_neg_inf_cubes_start;


        ShapesArraysMetadata {
            cubes_start,
            cubes_amount,
            spheres_start,
            spheres_amount,
            inf_cubes_start,
            inf_cubes_amount,
            sph_cubes_start,
            sph_cubes_amount,
            s_cubes_start,
            s_cubes_amount,
            s_spheres_start,
            s_spheres_amount,
            s_inf_cubes_start,
            s_inf_cubes_amount,
            s_sph_cubes_start,
            s_sph_cubes_amount,
            neg_cubes_start,
            neg_cubes_amount,
            neg_spheres_start,
            neg_spheres_amount,
            neg_inf_cubes_start,
            neg_inf_cubes_amount,
            neg_sph_cubes_start,
            neg_sph_cubes_amount,
            s_neg_cubes_start,
            s_neg_cubes_amount,
            s_neg_spheres_start,
            s_neg_spheres_amount,
            s_neg_inf_cubes_start,
            s_neg_inf_cubes_amount,
            s_neg_sph_cubes_start,
            s_neg_sph_cubes_amount,

            // padding_byte1: 0u32,
            // padding_byte2: 0u32,
            // undestroyable_cubes_start: 0u32,
            // undestroyable_cubes_amount: 0u32,
        }
    }


    fn update_spherical_areas_and_get_meatadata(
        &mut self,
        camera: &Camera,
        clip_planes: (Vec4, Vec4, Vec4, Vec4),
        stickiness_value: f32,
    ) -> (SphericalAreasMetadata, u32, u32)
    {
        let mut coloring_areas_start = 0u32;
        let mut coloring_areas_amount = 0u32;

        let mut volume_areas_start = 0u32;
        let mut volume_areas_amount = 0u32;

        let mut waves_start = 0u32;
        let mut waves_amount = 0u32;

        let mut index = 0;
        coloring_areas_start = 0u32;

        while let Some(area) = self.frame_coloring_areas_buffer.pop() {
            if check_if_player_see_sphere(
                camera,
                Vec4::from_array(area.pos),
                area.radius,
                clip_planes,
            )
            {
                self.spherical_areas_data[index] = area;

                index += 1;
            }
        }

        coloring_areas_amount = index as u32;

        volume_areas_start = index as u32;

        while let Some(area) = self.frame_spherical_volume_areas_buffer.pop() {
            if check_if_player_see_sphere(
                camera,
                Vec4::from_array(area.pos),
                area.radius,
                clip_planes,
            )
            {
                self.spherical_areas_data[index] = area;

                index += 1;
            }
        }

        volume_areas_amount = index as u32 - volume_areas_start;

        waves_start = index as u32;


        while let Some(area) = self.frame_waves_buffer.pop() {
            if check_if_player_see_sphere(
                camera,
                Vec4::from_array(area.pos),
                area.radius,
                clip_planes,
            )
            {
                self.spherical_areas_data[index] = area;

                index += 1;
            }
        }

        waves_amount = index as u32 - waves_start;


        (
            SphericalAreasMetadata {
                holegun_colorized_areas_start: coloring_areas_start,
                holegun_colorized_areas_amount: coloring_areas_amount,
                explode_areas_start: volume_areas_start,
                explode_areas_amount: volume_areas_amount,
                // empty_byte1: 0u32,
                // empty_byte2: 0u32,
            },
            waves_start,
            waves_amount
        )
    }


    fn update_beams_buffers_and_get_amount(&mut self) -> u32 {
        let mut index = 0_usize;

        while let Some(area) = self.frame_beam_volume_areas_buffer.pop() {
            self.beam_areas_data[index] = area;

            index += 1;
        }

        index as u32
    }
 

    fn update_player_forms_buffers_and_get_amount(
        &mut self,
        camera: &Camera,
        clip_planes: (Vec4, Vec4, Vec4, Vec4),
        stickiness_value: f32,
    ) -> u32
    {
        let mut index = 0_usize;

        while let Some(player_form) = self.frame_player_forms_buffer.pop()
        {
            self.player_forms_data[index] = player_form;

            index += 1;
        }

        index as u32
    }


    pub fn update(
        &mut self,
        world: &World,
        time: &TimeSystem,
        window: &Window,
        static_bounding_box: &BoundingBox,
        static_data: &StaticRenderData,
    ) {
        self.clear_all_frame_buffers();

        let main_camera =  world.actors
            .get(&world.main_player_id)
            .expect("World have wrong main_player id")
            .get_actor_as_controlled()
            .expect("Main actor is not ControlledActor")
            .get_camera();

        let screen_aspect = {
            let size = window.inner_size();
            size.width as f32 / size.height as f32
        };


        let clip_planes = get_view_clip_planes(&main_camera, screen_aspect);

        let dyn_bb = self.get_data_from_actors_visual_elements(world, static_bounding_box);

        let shapes_arrays_metadata = self.update_dynamic_shapes_buffers_and_get_metadata(
            static_data,

            &main_camera,
            clip_planes,
            world.level.all_shapes_stickiness_radius
        );

        let (spherical_areas_meatadata, waves_start, waves_amount) =
            self.update_spherical_areas_and_get_meatadata(
                &main_camera,
                clip_planes,
                world.level.all_shapes_stickiness_radius
            );

        let beams_areas_amount = self.update_beams_buffers_and_get_amount();

        let player_forms_amount = self.update_player_forms_buffers_and_get_amount(
            &main_camera,
            clip_planes,
            world.level.all_shapes_stickiness_radius
        );

        let players_screen_effects = get_players_screen_effects(world);

        self.other_dynamic_data.update(
            world,
            time,
            window,
            shapes_arrays_metadata,
            spherical_areas_meatadata,
            waves_start,
            waves_amount,
            beams_areas_amount,
            player_forms_amount,
            players_screen_effects,
            dyn_bb,
        );
    }
}



fn get_players_screen_effects(world: &World) -> &PlayerScreenEffects {
    world.actors
        .get(&world.main_player_id)
        .expect("Render system ERROR: World have not main player on main_player_id")
        .get_actor_as_controlled()
        .expect("Main actor is not ControlledActor")
        .get_screen_effects()
}


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PlayerProjectionForShader
{
    position: [f32;4],
    empty_byte1: f32,
    empty_byte2: f32,
    radius: f32,
    zw_offset: f32,
}

impl From<&PlayerProjection> for PlayerProjectionForShader
{
    fn from(value: &PlayerProjection) -> Self {
        let body = value.body.as_ref();

        match body {
            Some(projection_body) =>
            {
                PlayerProjectionForShader {
                    position: projection_body.position.to_array(),
                    empty_byte1: 0.0,
                    empty_byte2: 0.0,
                    radius: projection_body.radius,
                    zw_offset: projection_body.zx_rotation_offset,
                }                
            }
            None =>
            {
                PlayerProjectionForShader {
                    position: [0.0;4],
                    empty_byte1: 0.0,
                    empty_byte2: 0.0,
                    radius: 0.0,
                    zw_offset: 0.0,
                }
            }
        }
    }
}


impl Default for PlayerProjectionForShader
{
    fn default() -> Self {
        PlayerProjectionForShader {
            position: [0.0;4],
            empty_byte1: 0.0,
            empty_byte2: 0.0,
            radius: 0.0,
            zw_offset: 0.0
        }
    }
}


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct OtherDynamicData {
    dynamic_shapes_arrays_metadata: ShapesArraysMetadata,
    spherical_areas_metadata: SphericalAreasMetadata,
    camera_data: CameraUniform,

    waves_start: u32,
    waves_amount: u32,

    beam_areas_amount: u32,
    player_forms_amount: u32,

    player_projections: [PlayerProjectionForShader; 16],

    w_scanner_radius: f32,
    w_scanner_ring_intesity: f32,
    w_scanner_enemies_intesity: f32,

    death_screen_effect: f32,
    getting_damage_screen_effect: f32,
    splited_screen_in_2d_3d_example: f32,
    screen_aspect: f32,
    time: f32,

    additional_data: [f32;4],
    additional_data_2: [f32;4],
}

impl OtherDynamicData {
    pub fn update(
        &mut self,
        world: &World,
        time: &TimeSystem,
        window: &Window,
        shapes_arrays_metadata: ShapesArraysMetadata,
        spherical_areas_meatadata: SphericalAreasMetadata,
        waves_start: u32,
        waves_amount: u32,
        beams_areas_amount: u32,
        player_forms_amount: u32,
        players_screen_effects: &PlayerScreenEffects,
        frame_bounding_box: BoundingBox,
    ) {
        // let explore_w_pos;
        // let explore_w_coef;

        // self.additional_data = frame_bounding_box.pos_surfs.to_array();
        // self.additional_data_2 = frame_bounding_box.neg_surfs.to_array();

        let main_actor = world.actors
            .get(&world.main_player_id)
            .expect("World have not main player Actor");

        let camera = main_actor
            .get_actor_as_controlled()
            .expect("Main actor is not ControlledActor")
            .get_camera();

        // if it is 2d-3d example: send 3d slice transform data into the raymarch shader
        if let ActorWrapper::PlayerFor2d3dExample(player) = main_actor
        {
            self.additional_data = player.get_2d_slice_pos().to_array();
            self.additional_data_2 = player.get_2d_slice_xz_rot().to_cols_array();
            self.splited_screen_in_2d_3d_example = player.show_3d_example_current_value;
        }
        // self.explore_w_pos = explore_w_pos;
        // self.explore_w_coef = explore_w_coef;

        self.camera_data = CameraUniform {
            cam_pos: camera.get_position().to_array(),
            cam_zw_rot: camera.get_zw_rotation_matrix().to_cols_array(),
            cam_zy_rot: camera.get_zy_rotation_matrix().to_cols_array(),
            cam_zx_rot: camera.get_zx_rotation_matrix().to_cols_array(),
        };

        self.dynamic_shapes_arrays_metadata = shapes_arrays_metadata;
        self.spherical_areas_metadata = spherical_areas_meatadata;

        self.screen_aspect = {
            let size = window.inner_size();
            size.width as f32 / size.height as f32
        };

        self.beam_areas_amount = beams_areas_amount;

        self.time = time.timestamp_of_main_loop_start.elapsed().as_secs_f32();

        self.player_forms_amount = player_forms_amount;

        self.player_projections = [PlayerProjectionForShader::default(); 16];

        for (i, projection) in players_screen_effects.player_projections.iter().enumerate()
        {
            self.player_projections[i] = PlayerProjectionForShader::from(projection);
        }

        self.w_scanner_radius = {
            players_screen_effects.w_scanner_radius
        };
        self.w_scanner_ring_intesity = {
            if players_screen_effects.w_scanner_is_active {
                players_screen_effects.w_scanner_ring_intesity
            } else {
                0.0
            }
        };
        self.w_scanner_enemies_intesity = {
            players_screen_effects.w_scanner_enemies_intesity
        };

        self.death_screen_effect = players_screen_effects.death_screen_effect;
        self.getting_damage_screen_effect = players_screen_effects.getting_damage_screen_effect;

        self.waves_start = waves_start;
        self.waves_amount = waves_amount;
    }
}

impl Default for OtherDynamicData {
    fn default() -> Self {

        OtherDynamicData {
            dynamic_shapes_arrays_metadata: ShapesArraysMetadata::default(),
            spherical_areas_metadata: SphericalAreasMetadata::default(),
            camera_data: CameraUniform::default(),
            waves_start: 0u32,
            waves_amount: 0u32,
            beam_areas_amount: 0,
            player_forms_amount: 0,

            player_projections: [PlayerProjectionForShader::default(); 16],

            w_scanner_radius: 0.0,
            w_scanner_ring_intesity: 0.0,
            w_scanner_enemies_intesity: 0.0,

            death_screen_effect: 0.0,
            getting_damage_screen_effect: 0.0,
            splited_screen_in_2d_3d_example: 0.5,
            screen_aspect: 1.0,
            time: 0.0,
            additional_data: [0.0;4],
            additional_data_2: [0.0;4],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub cam_pos: [f32; 4],
    cam_zw_rot: [f32; 16],
    cam_zy_rot: [f32; 16],
    cam_zx_rot: [f32; 16],
}

impl Default for CameraUniform {
    fn default() -> Self {
        let cam_pos = [0.0, 0.0, 0.0, 0.0];
        let cam_zw_rot = Mat4::IDENTITY.to_cols_array();
        let cam_zy_rot = Mat4::IDENTITY.to_cols_array();
        let cam_zx_rot = Mat4::IDENTITY.to_cols_array();

        CameraUniform {
            cam_pos,
            cam_zw_rot,
            cam_zy_rot,
            cam_zx_rot,
        }
    }
}


pub struct SpecificShapeBuffers {
    pub normal: Vec<Shape>,
    pub negative: Vec<Shape>,
    pub stickiness: Vec<Shape>,
    pub neg_stickiness: Vec<Shape>,
}

impl Default for SpecificShapeBuffers {
    fn default() -> Self {
        SpecificShapeBuffers {
            normal: Vec::new(),
            negative: Vec::new(),
            stickiness: Vec::new(),
            neg_stickiness: Vec::new(),
        }
    }
}

impl SpecificShapeBuffers {
    pub fn new() -> Self {
        SpecificShapeBuffers {
            normal: Vec::new(),
            negative: Vec::new(),
            stickiness: Vec::new(),
            neg_stickiness: Vec::new(),
        }
    }

    pub fn clear_buffers(&mut self) {
        self.normal.clear();
        self.negative.clear();
        self.stickiness.clear();
        self.neg_stickiness.clear();
    }
}


pub fn check_if_player_see_cube(
    camera: &Camera,
    cube_pos: Vec4,
    cube_size: Vec4,
    planes: (Vec4, Vec4, Vec4, Vec4),

) -> bool
{
    let (
        up_plane,
        down_plane,
        left_plane,
        right_plane,
    ) = planes;
    
    cube_is_above_or_intersect_the_plane
    (
        cube_pos - camera.get_position(),
        cube_size,
        up_plane
    )
    &&
    cube_is_above_or_intersect_the_plane
    (
        cube_pos - camera.get_position(),
        cube_size,
        down_plane
    )
    &&
    cube_is_above_or_intersect_the_plane
    (
        cube_pos - camera.get_position(),
        cube_size,
        left_plane
    )
    &&
    cube_is_above_or_intersect_the_plane
    (
        cube_pos - camera.get_position(),
        cube_size,
        right_plane
    )
}


pub fn check_if_player_see_sphere(
    camera: &Camera,
    sphere_pos: Vec4,
    sphere_radius: f32,
    planes: (Vec4, Vec4, Vec4, Vec4),
) -> bool
{
    let (
        up_plane,
        down_plane,
        left_plane,
        right_plane,
    ) = planes;

    sphere_is_above_or_intersect_the_plane
    (
        sphere_pos - camera.get_position(),
        sphere_radius,
        up_plane
    )
    &&
    sphere_is_above_or_intersect_the_plane
    (
        sphere_pos - camera.get_position(),
        sphere_radius,
        down_plane
    )
    &&
    sphere_is_above_or_intersect_the_plane
    (
        sphere_pos - camera.get_position(),
        sphere_radius,
        left_plane
    )
    &&
    sphere_is_above_or_intersect_the_plane
    (
        sphere_pos - camera.get_position(),
        sphere_radius,
        right_plane
    )
}


pub fn cube_is_above_or_intersect_the_plane
(
    cube_pos: Vec4,
    cube_size: Vec4,
    plane: Vec4,
) -> bool
{
    assert!(
        cube_size.x > 0.0 &&
        cube_size.y > 0.0 &&
        cube_size.z > 0.0 &&
        cube_size.w > 0.0
    );

    (cube_pos + cube_size*Vec4::new(1.0, 1.0, 1.0, 1.0)).dot(plane) > 0.0
    ||
    (cube_pos + cube_size*Vec4::new(1.0, 1.0, 1.0, -1.0)).dot(plane) > 0.0
    ||
    (cube_pos + cube_size*Vec4::new(1.0, 1.0, -1.0, 1.0)).dot(plane) > 0.0
    ||
    (cube_pos + cube_size*Vec4::new(1.0, 1.0, -1.0, -1.0)).dot(plane) > 0.0
    ||
    (cube_pos + cube_size*Vec4::new(1.0, -1.0, 1.0, 1.0)).dot(plane) > 0.0
    ||
    (cube_pos + cube_size*Vec4::new(1.0, -1.0, 1.0, -1.0)).dot(plane) > 0.0
    ||
    (cube_pos + cube_size*Vec4::new(1.0, -1.0, -1.0, 1.0)).dot(plane) > 0.0
    ||
    (cube_pos + cube_size*Vec4::new(1.0, -1.0, -1.0, -1.0)).dot(plane) > 0.0
    ||
    (cube_pos + cube_size*Vec4::new(-1.0, 1.0, 1.0, 1.0)).dot(plane) > 0.0
    ||
    (cube_pos + cube_size*Vec4::new(-1.0, 1.0, 1.0, -1.0)).dot(plane) > 0.0
    ||
    (cube_pos + cube_size*Vec4::new(-1.0, 1.0, -1.0, 1.0)).dot(plane) > 0.0
    ||
    (cube_pos + cube_size*Vec4::new(-1.0, 1.0, -1.0, -1.0)).dot(plane) > 0.0
    ||
    (cube_pos + cube_size*Vec4::new(-1.0, -1.0, 1.0, 1.0)).dot(plane) > 0.0
    ||
    (cube_pos + cube_size*Vec4::new(-1.0, -1.0, 1.0, -1.0)).dot(plane) > 0.0
    ||
    (cube_pos + cube_size*Vec4::new(-1.0, -1.0, -1.0, 1.0)).dot(plane) > 0.0
    ||
    (cube_pos + cube_size*Vec4::new(-1.0, -1.0, -1.0, -1.0)).dot(plane) > 0.0
}


pub fn sphere_is_above_or_intersect_the_plane
(
    sphere_pos: Vec4,
    sphere_radius: f32,
    plane: Vec4,
) -> bool
{
    // assert!(
    //     sphere_radius >= 0.0
    // );

    (plane).dot(sphere_pos + (plane*sphere_radius)) > 0.0
}


pub fn get_view_clip_planes(
    camera: &Camera,
    screen_aspect: f32,
) -> (Vec4, Vec4, Vec4, Vec4)
{
    let cam_zw_rot = camera.get_zw_rotation_matrix();
    let cam_zy_rot = camera.get_zy_rotation_matrix();
    let cam_zx_rot = camera.get_zx_rotation_matrix();

    let rotation = camera.get_rotation_matrix().inverse();
    
    let up_clip_plane = Vec4::new(0.0, -1.428573, -1.0, 0.0).normalize();
    let up_clip_plane = rotation * up_clip_plane;

    let down_clip_plane = Vec4::new(0.0, 1.428573, -1.0, 0.0).normalize();
    let down_clip_plane = rotation * down_clip_plane;

    let (left_clip_plane, right_clip_plane) = {
        let x = (90.0 - (0.7*screen_aspect).atan().to_degrees()).to_radians().tan();

        (
            rotation * (Vec4::new(x, 0.0, -1.0, 0.0).normalize()),
            rotation * (Vec4::new(-x, 0.0, -1.0, 0.0).normalize())
        )
    };

    (
        up_clip_plane,
        down_clip_plane,
        left_clip_plane,
        right_clip_plane
    )
}