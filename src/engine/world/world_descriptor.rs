// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use super::{World, WorldConfig};
use crate::engine::shared_systems::EngineSharedSystems;
use fyrox_core::parking_lot::Mutex;
use crate::engine::shared_systems::resources::{ResourceSystem, ResourceLoader};
use std::sync::{Arc, OnceLock};
use fyrox_resource::Resource;
use crate::engine::shared_systems::resources::graphics::{CubemapTexture, LevelResource, RenderPipelineResource, BuiltinPipelineType, Skybox};
use std::collections::HashMap;

use crate::{
    actor::{
        ActorWrapper, droped_rotator_tool::DropedRotatorTool, final_trigger::FinalTrgger, flag::Flag, flag_base::FlagBase, main_player::MainPlayer, mover_w::MoverW, new_spawn_area::NewSpawnArea, obstacle_course_free_movement_player::ObstacleCourseFreeMovementPlayer, session_controller::SessionController, trgger_orb::TriggerOrb, trigger::Trigger, triggering_special_lift::TriggeringSpecialLiftActor, triggering_wandering_actor::TriggeringWanderingActor, wandering_actor::{
            WanderingActor,
            WanderingActorMovementType,
        }
    }, engine::{
        world::physics::{
            physics_system_data::ShapeType, static_collider::StaticCollider
        }, shared_systems::render::{raymarch_shader_generator::{AllowedDynamicObjectsInBspTreeTable, BspTree}, render_data::static_render_data::StaticRenderData}, world::static_object::{
            ObjectMaterial, StaticObject
        }
    }, engine::transform::Transform
};

use net_protocol::gameplay::Team;
use glam::{Vec4, Vec3};
use serde_json::Value;
use super::static_object::{WFloor, WRoof};



#[derive(Clone, Copy, Debug)]
pub struct EnvironmentVisualSettings {
    pub sky_color: Vec4,
    pub fog_color: Vec4,
    pub fresnel_color: Vec4,
    pub neon_wireframe_color: Vec4,
    pub sun_color: Vec4,
    pub sun_direction: Vec4,
    pub red_map_color: Vec4,
    pub blue_map_color: Vec4,
}

pub struct DefaultStaticObjectSettings {
    friction: f32,
    bounce_rate: f32,
    default_material_index: i32,
    roundness: f32,
    stickiness: bool,
    is_positive: bool,
    undestroyable: bool,
}

/// Immutable parsed level, shared by world preparation and geometry-specialized shaders.
pub struct LevelData {
    pub(in crate::engine) source_origin: String,
    pub shader: BuiltinPipelineType,
    pub skybox: Skybox,
    pub with_upscale: bool,
    pub allowed_dyn_objects: AllowedDynamicObjectsInBspTreeTable,
    static_data: OnceLock<Arc<StaticRenderData>>,
    bsp: OnceLock<BspTree>,
    pub name: String,
    pub static_objects: Vec<StaticObject>,
    pub all_shapes_stickiness_radius: f32,
    pub visual_materials: Vec<ObjectMaterial>,
    pub blue_players_visual_materials: (i32, i32),
    pub red_players_visual_materials: (i32, i32),
    pub visual_settings_of_environment: EnvironmentVisualSettings,
    pub red_base_position: Vec4,
    pub blue_base_position: Vec4,

    main_actor_description: Value,
    actor_descriptions: Value,
    default_settings: DefaultStaticObjectSettings,
    materials_table: HashMap<String, i32>,
    it_is_2d_3d_example: bool,
}

impl LevelData {
    pub fn static_render_data(&self) -> &Arc<StaticRenderData> {
        self.static_data.get_or_init(|| {
            let mut data = StaticRenderData::new();
            data.update(self);
            Arc::new(data)
        })
    }

    pub fn bsp(&self) -> &BspTree {
        self.bsp.get_or_init(|| BspTree::new(self.static_render_data()))
    }

    fn build_actors(&self, resources: &mut ResourceLoader) -> (ActorWrapper, Vec<ActorWrapper>) {
        let main = parse_json_main_actor(&self.main_actor_description, resources)
            .expect("Wrong JSON map format: missing main actor");
        (main, parse_json_actors(&self.actor_descriptions, &self.default_settings,
            &self.materials_table, self.it_is_2d_3d_example, resources))
    }
}

pub struct WorldDescriptor {
    pub level: Arc<LevelData>,
    pub level_key: String,
    pub gpu: Option<WorldDescriptorGpu>,
    pub(super) source: Resource<LevelResource>,
    ready_actors: Mutex<(ActorWrapper, Vec<ActorWrapper>)>,
}

impl std::ops::Deref for WorldDescriptor {
    type Target = LevelData;
    fn deref(&self) -> &LevelData { &self.level }
}

pub struct WorldDescriptorGpu {
    pub main: Resource<RenderPipelineResource>,
    pub sky: Resource<CubemapTexture>,
}

impl WorldDescriptor {
    pub(in crate::engine) async fn prepare(
        level_key: String, source: Resource<LevelResource>, gpu: Option<WorldDescriptorGpu>,
        resources: Arc<ResourceSystem>,
    ) -> Result<Self, String> {
        let level = source.data_ref().value().clone();
        let build = level.clone();
        let (actors, dependencies) = tokio::task::spawn_blocking(move || {
            let mut dependencies = Vec::new();
            let actors = build.build_actors(&mut resources.loader_with_dependencies(&mut dependencies));
            (actors, dependencies)
        }).await.map_err(|e| e.to_string())?;
        for dep in dependencies {
            dep.await.map_err(|e| format!("Actor resource failed: {e:?}"))?;
        }
        if let Some(gpu) = &gpu {
            gpu.main.clone().await.map_err(|e| format!("Scene pipeline failed: {e:?}"))?;
            gpu.sky.clone().await.map_err(|e| format!("Skybox failed: {e:?}"))?;
        }
        Ok(Self { level, level_key, gpu, source, ready_actors: Mutex::new(actors) })
    }

    pub(in crate::engine) fn take_actors(&self, resources: &mut ResourceLoader) -> (ActorWrapper, Vec<ActorWrapper>) {
        let next = self.level.build_actors(resources);
        std::mem::replace(&mut *self.ready_actors.lock(), next)
    }

    /// Refill the spare from the warm cache before publishing the world. Callbacks run
    /// after the swap, so they may request this descriptor again.
    pub fn give_me_world(&self, config: WorldConfig, shared: &mut EngineSharedSystems) -> World {
        let (main, actors) = self.take_actors(&mut shared.resources);
        World::activate(self, main, actors, config, shared)
    }
}

pub(in crate::engine) fn parse_json_level(json: Value, source_origin: String) -> LevelData
{
    let json_level = json
    .as_object()
    .expect("Wrong JSON map format: the root JSON value of a level must be an object");

    let level_name: String = {
        json_level
            .get("level_name")
            .expect("Wrong JSON map format: JSON level must have a level_name property")
            .as_str()
            .expect("Wrong JSON map format: level_name value must be a string")
            .into()
    };

    let all_shapes_stickiness_radius: f32 = {
        json_level
            .get("all_shapes_stickiness_radius")
            .expect("Wrong JSON map format: JSON level must have an all_shapes_stickiness_radius property")
            .as_f64()
            .expect("Wrong JSON map format: all_shapes_stickiness_radius value must be a float number")
            as f32
    };

    let (visual_materials, materials_table) = {
        let json_visual_materials = json_level
            .get("visual_materials")
            .expect("Wrong JSON map format: JSON level must have a visual_materials property");

        parse_visual_materials(json_visual_materials)
    };

    let default_settings = {
        let json_defaults = json_level
            .get("defaults")
            .expect("Wrong JSON map format: JSON level must have a defaults property");

        parse_json_defaults(json_defaults, &materials_table)
    };

    let red_players_visual_materials = {
        let json_players_visual_materials = json_level
            .get("red_players_visual_materials")
            .expect("Wrong JSON map format: JSON level must have a red_players_visual_materials property");

        parse_players_visual_materials(json_players_visual_materials, &materials_table)
    };

    let blue_players_visual_materials = {
        let json_players_visual_materials = json_level
            .get("blue_players_visual_materials")
            .expect("Wrong JSON map format: JSON level must have a blue_players_visual_materials property");

        parse_players_visual_materials(json_players_visual_materials, &materials_table)
    };

    let static_objects = {
        let json_static_objects = json_level
            .get("static_objects")
            .expect("Wrong JSON map format: JSON level must have a static_objects property");

        parse_json_static_objects(json_static_objects, &default_settings, &materials_table)
    };

    let actor_descriptions = json_level
        .get("actors")
        .expect("Wrong JSON map format: JSON level must have an actors property");

    let main_actor_description = json_level
        .get("main_actor")
        .expect("Wrong JSON map format: JSON level must have a main_actor property");

    let visual_settings_of_environment = {
        let json_visual_settings_of_environment = json_level
            .get("visual_settings_of_environment")
            .expect("Wrong JSON map format: JSON level must have a visual_settings_of_environment property");

        parse_json_visual_settings_of_environment(json_visual_settings_of_environment)
    };

    let red_base_position = {
        let red_base_position_json = json_level
            .get("red_base_position")
            .expect("Wrong JSON map format: JSON level must have a red_base_position property");

        parse_json_into_transform(red_base_position_json, "red_base_position").get_position()
    };
    
    let blue_base_position = {
        let blue_base_position_json = json_level
            .get("blue_base_position")
            .expect("Wrong JSON map format: JSON level must have a blue_base_position property");

        parse_json_into_transform(blue_base_position_json, "blue_base_position").get_position()
    };

    if let Some(value) = json_level.get("generated_shader_with_bsp_tree") {
        assert_eq!(value.as_bool(), Some(true), "Wrong JSON map format: only BSP scene shaders are supported");
    }

    let it_is_2d_3d_example = {
        json_level
            .get("it_is_2d_3d_example")
            .expect("Wrong JSON map format: JSON level must have an it_is_2d_3d_example property")
            .as_bool()
            .expect("Wrong JSON map format: it_is_2d_3d_example property must be a boolean")
    };

    let allowed_dyn_objects = {
        let allowed_dynamic_objects_in_bsp_tree = json_level
            .get("allowed_dynamic_objects_in_bsp_tree")
            .expect("Wrong JSON map format: JSON level must have an allowed_dynamic_objects_in_bsp_tree property");

        parse_allowed_dynamic_objects_in_bsp_tree(&allowed_dynamic_objects_in_bsp_tree)
    };

    let shader = BuiltinPipelineType::parse(json_level.get("main_shader").and_then(Value::as_str)
        .expect("Wrong JSON map format: main_shader must be a string"))
        .filter(|kind| matches!(kind, BuiltinPipelineType::Slice | BuiltinPipelineType::Obstacle | BuiltinPipelineType::DitheredObstacle))
        .expect("Wrong JSON map format: unknown main_shader");
    let skybox = Skybox::parse(&json_level["visual_settings_of_environment"])
        .unwrap_or_else(|err| panic!("Wrong JSON map format: {err}"));
    let with_upscale = json_level.get("with_upscale").map(|v| v.as_bool()
        .expect("Wrong JSON map format: with_upscale must be a boolean")).unwrap_or(true);
    LevelData {
        source_origin, shader, skybox, with_upscale, allowed_dyn_objects,
        bsp: OnceLock::new(), static_data: OnceLock::new(),
        name: level_name,
        static_objects,
        all_shapes_stickiness_radius,
        visual_materials,
        blue_players_visual_materials,
        red_players_visual_materials,
        visual_settings_of_environment,
        red_base_position,
        blue_base_position,
        main_actor_description: main_actor_description.clone(),
        actor_descriptions: actor_descriptions.clone(),
        default_settings,
        materials_table,
        it_is_2d_3d_example,
    }
}

#[allow(dead_code)] // game content that is not wired into the current maps/modes; kept for later
fn parse_mover_w(json_obj: &Value, w_levels: &Vec<f32>) -> MoverW
{
    let transform = parse_json_into_transform(json_obj, "mover_w");

    let direction = {
        json_obj.get("direction")
            .expect("Wrong JSON map format: mover_w has no direction property")
            .as_f64()
            .expect("Wrong JSON map format: mover_w's direction property must be a number")
            as f32
    };

    MoverW::new(transform.get_position(), direction, w_levels)
}


fn parse_json_visual_settings_of_environment(
    json: &Value,
) -> EnvironmentVisualSettings {
    let obj = json
        .as_object()
        .expect("Wrong JSON map format: the visual_settings_of_environment property must be an object");

    let sky_color = {
        let json_value = obj.get("sky_color")
            .expect("Wrong JSON map format: visual_settings_of_environment has no sky_color property");

        parse_json_color_and_multiplier(json_value)
    };

    let fog_color = {
        let json_value = obj.get("fog_color")
            .expect("Wrong JSON map format: visual_settings_of_environment has no fog_color property");

        parse_json_color_and_multiplier(json_value)
    };

    let fresnel_color = {
        let json_value = obj.get("frenel_color")
            .expect("Wrong JSON map format: visual_settings_of_environment has no fresnel_color property");

        parse_json_color_and_multiplier(json_value)
    };

    let neon_wireframe_color = {
        let json_value = obj.get("neon_wireframe_color")
            .expect("Wrong JSON map format: visual_settings_of_environment has no neon_wireframe_color property");

        parse_json_color_and_multiplier(json_value)
    };

    let sun_color = {
        let json_value = obj.get("sun_color")
            .expect("Wrong JSON map format: visual_settings_of_environment has no sun_color property");

        parse_json_color_and_multiplier(json_value)
    };

    let sun_direction = {
        let json_value = obj.get("sun_direction")
            .expect("Wrong JSON map format: visual_settings_of_environment has no sun_direction property");

        let x = json_value
            .get("x")
            .expect("Wrong JSON map format: sun_direction has no x property")
            .as_f64()
            .expect("Wrong JSON map format: the x property of sun_direction must be a float number");
    
        let y = json_value
            .get("y")
            .expect("Wrong JSON map format: sun_direction has no y property")
            .as_f64()
            .expect("Wrong JSON map format: the y property of sun_direction must be a float number");
    
        let z = json_value
            .get("z")
            .expect("Wrong JSON map format: sun_direction has no z property")
            .as_f64()
            .expect("Wrong JSON map format: the z property of sun_direction must be a float number");
    
        let w = json_value
            .get("w")
            .expect("Wrong JSON map format: sun_direction has no w property")
            .as_f64()
            .expect("Wrong JSON map format: the w property of sun_direction must be a float number");
        
        Vec4::new(x as f32, y as f32, z as f32, w as f32)
    };

    let red_map_color = {
        let json_value = obj.get("red_map_color")
            .expect("Wrong JSON map format: visual_settings_of_environment has no red_map_color property");

        parse_json_color_and_multiplier(json_value)
    };

    let blue_map_color = {
        let json_value = obj.get("blue_map_color")
            .expect("Wrong JSON map format: visual_settings_of_environment has no blue_map_color property");

        parse_json_color_and_multiplier(json_value)
    };

    EnvironmentVisualSettings {
        sky_color,
        sun_color,
        sun_direction,
        fog_color,
        fresnel_color,
        neon_wireframe_color,
        red_map_color,
        blue_map_color,
    }
}

fn parse_json_color_and_multiplier(
    json: &Value,
) -> Vec4 {
    let obj = json
        .as_object()
        .expect("Wrong JSON map format: every color in visual_settings_of_environment must be an object");

    let red = {
        obj
            .get("red")
            .expect("Wrong JSON map format: color must have a red property")
            .as_f64()
            .expect("Wrong JSON map format: red value must be a float number")
            as f32
    };

    let green = {
        obj
            .get("green")
            .expect("Wrong JSON map format: color must have a green property")
            .as_f64()
            .expect("Wrong JSON map format: green value must be a float number")
            as f32
    };

    let blue = {
        obj
            .get("blue")
            .expect("Wrong JSON map format: color must have a blue property")
            .as_f64()
            .expect("Wrong JSON map format: blue value must be a float number")
            as f32
    };

    let mult = {
        obj
            .get("multiplier")
            .expect("Wrong JSON map format: color must have a multiplier property")
            .as_f64()
            .expect("Wrong JSON map format: multiplier value must be a float number")
            as f32
    };

    Vec4::new(red, green, blue, 0.0) * mult
}


fn parse_players_visual_materials(
    json: &Value,
    materials_table: &HashMap<String, i32>
) -> (i32, i32) {
    let obj = json
        .as_object()
        .expect("Wrong JSON map format: the players_visual_materials property must be an object");

    let inner_material_name = obj
        .get("inner")
        .expect("players_visual_materials has no inner property")
        .as_str()
        .expect("inner property in players_visual_materials is not a string")
        .to_string();

    let inner_material_index = materials_table
        .get(&inner_material_name)
        .expect("the inner material of players_visual_materials does not exist")
        .clone();

    let outer_material_name = obj
        .get("outer")
        .expect("players_visual_materials has no outer property")
        .as_str()
        .expect("outer property in players_visual_materials is not a string")
        .to_string();

    let outer_material_index = materials_table
        .get(&outer_material_name)
        .expect("the outer material of players_visual_materials does not exist")
        .clone();

    (inner_material_index, outer_material_index)
}


#[allow(dead_code)] // game content that is not wired into the current maps/modes; kept for later
fn parse_w_cups_visual_materials(
    json: &Value,
    materials_table: &HashMap<String, i32>
) -> i32 {
    let material_name = json
        .as_str()
        .expect("Wrong JSON map format: the w_cups_visual_materials property must be a string")
        .to_string();

    let material_index = materials_table
        .get(&material_name)
        .expect("the material of w_cups_visual_materials does not exist")
        .clone();

    material_index
}


fn parse_visual_materials(
    json: &Value
) -> (Vec<ObjectMaterial>, HashMap<String, i32>) {
    let mut visual_materials = Vec::new();
    let mut materials_table = HashMap::new();

    let array = json
        .as_array()
        .expect("Wrong JSON map format: the visual_materials property must be an array");

    let mut index = 0i32;

    for json_material in array {
        let (material, name) = parse_material(json_material);

        visual_materials.push(material);

        materials_table.insert(name, index);

        index += 1;
    }

    (visual_materials, materials_table)
}


fn parse_material(
    json: &Value
) -> (ObjectMaterial, String) {
    let obj = json
        .as_object()
        .expect("Wrong JSON map format: every material in visual_materials must be an object");

    let name = obj.get("name")
        .expect("a material in visual_materials has no name property")
        .as_str()
        .expect("the name property of a visual material must be a string")
        .to_string();

        let color = obj
                .get("color")
                .expect("Wrong JSON map format: material must have a color property")
                .as_object()
                .expect("Wrong JSON map format: color value must be an object");
    
        let red = {
            color
                .get("red")
                .expect("Wrong JSON map format: color must have a red property")
                .as_f64()
                .expect("Wrong JSON map format: red value must be a float number")
                as f32
        };
    
        let green = {
            color
                .get("green")
                .expect("Wrong JSON map format: color must have a green property")
                .as_f64()
                .expect("Wrong JSON map format: green value must be a float number")
                as f32
        };
    
        let blue = {
            color
                .get("blue")
                .expect("Wrong JSON map format: color must have a blue property")
                .as_f64()
                .expect("Wrong JSON map format: blue value must be a float number")
                as f32
        };
    
        let color = Vec3::new(red, green, blue);

        let roughness = {
            obj
                .get("roughness")
                .expect("Wrong JSON map format: material must have a roughness property")
                .as_f64()
                .expect("Wrong JSON map format: roughness value must be a number")
                as f32
        };
    
        let material = ObjectMaterial::new(color, roughness);

        (material, name)
}


#[allow(dead_code)] // game content that is not wired into the current maps/modes; kept for later
fn parse_w_cups(
    json: &Value
) -> (Option<WFloor>, Option<WRoof>) {
    let obj = json
        .as_object()
        .expect("Wrong JSON map format: the w_cups property must be an object");

    let w_floor = {
        if let Some(val) = obj.get("w_floor") {
            let val = val
                .as_f64()
                .expect("Wrong JSON map format: the w_floor property of w_cups must be a number")
                as f32;
            
            Some(WFloor {
                w_pos: val
            })
        } else {
            None
        }
    };

    let w_roof = {
        if let Some(val) = obj.get("w_roof") {
            let val = val
                .as_f64()
                .expect("Wrong JSON map format: the w_roof property of w_cups must be a number")
                as f32;
            
            Some(WRoof {
                w_pos: val
            })
        } else {
            None
        }
    };

    (w_floor, w_roof)
}


fn parse_json_defaults(
    json: &Value,
    materials_table: &HashMap<String, i32>
) -> DefaultStaticObjectSettings {
    let json_settings = json
        .as_object()
        .expect("Wrong JSON map format: the root JSON value of a level must be an object");

    let friction = {
        json_settings
            .get("friction")
            .expect("Wrong JSON map format: JSON defaults must have a friction property")
            .as_f64()
            .expect("Wrong JSON map format: friction value must be a float number")
            as f32
    };

    let bounce_rate = {
        json_settings
            .get("bounce_rate")
            .expect("Wrong JSON map format: JSON defaults must have a bounce_rate property")
            .as_f64()
            .expect("Wrong JSON map format: bounce_rate value must be a float number")
            as f32
    };

    let roundness = {
        json_settings
            .get("roundness")
            .expect("Wrong JSON map format: JSON defaults must have a roundness property")
            .as_f64()
            .expect("Wrong JSON map format: roundness value must be a float number")
            as f32
    };

    let stickiness = {
        json_settings
            .get("stickiness")
            .expect("Wrong JSON map format: JSON defaults must have a stickiness property")
            .as_bool()
            .expect("Wrong JSON map format: stickiness value must be a boolean")
    };

    let is_positive = {
        json_settings
            .get("is_positive")
            .expect("Wrong JSON map format: JSON defaults must have an is_positive property")
            .as_bool()
            .expect("Wrong JSON map format: is_positive value must be a boolean")
    };

    let visual_material_name = json_settings
        .get("visual_material")
        .expect("Wrong JSON map format: JSON defaults must have a visual_material property")
        .as_str()
        .expect("Wrong JSON map format: visual_material value must be an object")
        .to_string();
    
    let default_material_index = materials_table
        .get(&visual_material_name)
        .expect("the default visual_material in defaults has a wrong material name")
        .clone();

    let undestroyable = {
        json_settings
            .get("undestroyable")
            .expect("Wrong JSON map format: JSON defaults must have an undestroyable property")
            .as_bool()
            .expect("Wrong JSON map format: undestroyable value must be a boolean")
    };

    DefaultStaticObjectSettings {
        friction,
        bounce_rate,
        default_material_index,
        roundness,
        stickiness,
        is_positive,
        undestroyable
    }
}



fn parse_json_static_objects(
    json_map: &Value,
    defaults: &DefaultStaticObjectSettings,
    materials_table: &HashMap<String, i32>,
) -> Vec<StaticObject>
{
    let mut static_objects: Vec<StaticObject> = Vec::with_capacity(100);

    let array = json_map
        .as_array()
        .expect("Wrong JSON map format: static_objects value must be an array");

    for object in array {
        
        let object = object
            .as_object()
            .expect("Wrong JSON map format: every shape must be a JSON object");


        for (name, shape) in object {
            let static_object = parse_json_specific_shape(
                name, shape, defaults, materials_table
            );
            static_objects.push(static_object)
        }
    }

    static_objects
}



fn parse_json_specific_shape(
    shape_name: &String,
    json_shape: &Value,
    defaults: &DefaultStaticObjectSettings,
    materials_table: &HashMap<String, i32>,
) -> StaticObject {
    let shape_name = shape_name.as_str();

    let shape_type = {
        match shape_name {
            "cube" => {
                ShapeType::Cube
            },
            "cube_w_inf" => {
                panic!("cube_w_inf shapes are temporarily unsupported")
                // ShapeType::CubeInfW
            },
            "sphere" => {
                ShapeType::Sphere
            },
            "sph_cube" => {
                ShapeType::SphCube
            },
            _ => {
                panic!("Wrong JSON map format: unexpected shape {} in map", shape_name)
            }
        }
    };

    let position = parse_json_into_transform(json_shape, shape_name).get_position();

    let size = parse_json_into_size(json_shape, shape_name);

    let is_positive = parse_json_into_is_positive(
        json_shape, shape_name
    ).unwrap_or(defaults.is_positive);

    let bounce_rate = parse_json_into_bounce_rate(
        json_shape, shape_name
    ).unwrap_or(defaults.bounce_rate);

    let friction = parse_json_into_friction(
        json_shape, shape_name
    ).unwrap_or(defaults.friction);

    let roundness = parse_json_into_roundness(
        json_shape, shape_name
    ).unwrap_or(defaults.roundness);

    let stickiness = parse_json_into_stickiness(
        json_shape, shape_name
    ).unwrap_or(defaults.stickiness);

    let material_index = parse_json_into_material_index(
        json_shape, shape_name, materials_table
    ).unwrap_or(defaults.default_material_index);

    let undestroyable = parse_json_into_undestroyable(
        json_shape, shape_name
    ).unwrap_or(defaults.undestroyable);

    let collider = StaticCollider {
        shape_type,
        position,
        size,
        is_positive,
        friction,
        roundness,
        bounce_rate,
        stickiness,
        actor_id: None,
        undestroyable,
    };

    StaticObject {
        collider,
        material_index,
    }
}



fn parse_json_into_transform(shape: &Value, shape_name: &str) -> Transform {

    let shape = shape
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format: every shape must be a JSON object in {}",
                shape_name
            )
        );

    let json_transform = shape
        .get("transform")
        .expect(
            &format!
            (
                "Wrong JSON map format: no transform property in {}",
                shape_name
            )
        );

    let json_transform = json_transform
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format: transform property must be a JSON object in {}",
                shape_name
            )
        );

    let json_position = json_transform
        .get("position")
        .expect(
            &format!
            (
                "Wrong JSON map format: transform has no position property in {}",
                shape_name
            )
        )
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format: position property must be a JSON object in {}",
                shape_name
            )
        );
            

    let x = json_position
        .get("x")
        .expect(
            &format!
            (
                "Wrong JSON map format: transform has no x property in {}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format: the x property of transform must be a float number in {}",
                shape_name
            )
        );

    let y = json_position
        .get("y")
        .expect(
            &format!
            (
                "Wrong JSON map format: transform has no y property in {}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format: the y property of transform must be a float number in {}",
                shape_name
            )
        );

    let z = json_position
        .get("z")
        .expect(
            &format!
            (
                "Wrong JSON map format: transform has no z property in {}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format: the z property of transform must be a float number in {}",
                shape_name
            )
        );

    let w = json_position
        .get("w")
        .expect(
            &format!
            (
                "Wrong JSON map format: transform has no w property in {}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format: the w property of transform must be a float number in {}",
                shape_name
            )
        );
    
    let position = Vec4::new(x as f32, y as f32, z as f32, w as f32);

    let json_scale = json_transform
        .get("scale");

    if json_scale.is_none() {

        return Transform::from_position(position);
    }

    let json_scale = json_scale.expect(
        &format!
        (
            "Wrong JSON map format: transform has no scale property in {}",
            shape_name
        )
    )
    .as_object()
    .expect(
        &format!
        (
            "Wrong JSON map format: scale property must be a JSON object in {}",
            shape_name
        )
    );
            
    let x = json_scale
        .get("x")
        .expect(
            &format!
            (
                "Wrong JSON map format: transform has no x property in {}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format: the x property of transform must be a float number in {}",
                shape_name
            )
        );

    let y = json_scale
        .get("y")
        .expect(
            &format!
            (
                "Wrong JSON map format: transform has no y property in {}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format: the y property of transform must be a float number in {}",
                shape_name
            )
        );

    let z = json_scale
        .get("z")
        .expect(
            &format!
            (
                "Wrong JSON map format: transform has no z property in {}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format: the z property of transform must be a float number in {}",
                shape_name
            )
        );

    let w = json_scale
        .get("w")
        .expect(
            &format!
            (
                "Wrong JSON map format: transform has no w property in {}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format: the w property of transform must be a float number in {}",
                shape_name
            )
        );
    
    let scale = Vec4::new(x as f32, y as f32, z as f32, w as f32);
    
    Transform::from_position_and_scale(position, scale)
}



fn parse_json_into_size(shape: &Value, shape_name: &str) -> Vec4 {

    let shape = shape
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format: every shape must be a JSON object in {}",
                shape_name
            )
        );

    let json_size = shape
        .get("size")
        .expect(
            &format!
            (
                "Wrong JSON map format: no size property in {}",
                shape_name
            )
        );

    let json_size = json_size
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format: size property must be a JSON object in {}",
                shape_name
            )
        );

    let x = json_size
        .get("x")
        .expect(
            &format!
            (
                "Wrong JSON map format: size has no x property in {}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format: the x property of size must be a float number in {}",
                shape_name
            )
        );

    let y;
    let y_obj = json_size
        .get("y");
    
    if let Some(y_json) = y_obj {
        y = y_json
            .as_f64()
            .expect(
                &format!
                (
                    "Wrong JSON map format: the y property of size must be a float number in {}",
                    shape_name
                )
            );
    } else {

        return Vec4::new(x as f32, 0., 0., 0.);
    }

    let z;
    let z_obj = json_size
        .get("z");
    
    if let Some(z_json) = z_obj {
        z = z_json
            .as_f64()
            .expect(
                &format!
                (
                    "Wrong JSON map format: the z property of size must be a float number in {}",
                    shape_name
                )
            );
    } else {

        return Vec4::new(x as f32, y as f32, 0., 0.);
    }

    let w;
    let w_obj = json_size
        .get("w");
    
    if let Some(w_json) = w_obj {
        w = w_json
            .as_f64()
            .expect(
                &format!
                (
                    "Wrong JSON map format: the w property of size must be a float number in {}",
                    shape_name
                )
            );
    } else {
        return Vec4::new(x as f32, y as f32, z as f32, 0.);
    }

    Vec4::new(x as f32, y as f32, z as f32, w as f32)
}



fn parse_json_into_is_positive(json_shape: &Value, shape_name: &str) -> Option<bool> {

    let shape = json_shape
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format: every shape must be a JSON object in {}",
                shape_name
            )
        );

    let json_is_positive = shape.get("is_positive");

    if json_is_positive.is_none() {
        return None
    }

    let is_positive = json_is_positive
        .unwrap()
        .as_bool()
        .expect(
            &format!
            (
                "Wrong JSON map format: is_positive property must be a boolean in {}",
                shape_name
            )
        );
    
    Some(is_positive)
}



fn parse_json_into_undestroyable(json_shape: &Value, shape_name: &str) -> Option<bool> {

    let shape = json_shape
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format: every shape must be a JSON object in {}",
                shape_name
            )
        );

    let json_undestroyable = shape.get("undestroyable");

    if json_undestroyable.is_none() {
        return None
    }

    let undestroyable = json_undestroyable
        .unwrap()
        .as_bool()
        .expect(
            &format!
            (
                "Wrong JSON map format: undestroyable property must be a boolean in {}",
                shape_name
            )
        );
    
    Some(undestroyable)
}


fn parse_json_into_friction(json_shape: &Value, shape_name: &str) -> Option<f32> {

    let shape = json_shape
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format: every shape must be a JSON object in {}",
                shape_name
            )
        );

    let json_friction = shape.get("friction");

    if json_friction.is_none() {
        return None
    }

    let friction = json_friction
        .unwrap()
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format: friction property must be a float number in {}",
                shape_name
            )
        ) as f32;
    
    Some(friction)
}



fn parse_json_into_bounce_rate(json_shape: &Value, shape_name: &str) -> Option<f32> {

    let shape = json_shape
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format: every shape must be a JSON object in {}",
                shape_name
            )
        );

    let json_bounce_rate = shape.get("bounce_rate");

    if json_bounce_rate.is_none() {
        return None
    }

    let bounce_rate = json_bounce_rate
        .unwrap()
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format: bounce_rate property must be a float number in {}",
                shape_name
            )
        ) as f32;
    
    Some(bounce_rate)
}



fn parse_json_into_roundness(json_shape: &Value, shape_name: &str) -> Option<f32> {

    let shape = json_shape
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format: every shape must be a JSON object in {}",
                shape_name
            )
        );

    let json_roundness = shape.get("roundness");

    if json_roundness.is_none() {
        return None
    }

    let roundness = json_roundness
        .unwrap()
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format: roundness property must be a float number in {}",
                shape_name
            )
        ) as f32;
    
    Some(roundness)
}



fn parse_json_into_stickiness(json_shape: &Value, shape_name: &str) -> Option<bool> {

    let shape = json_shape
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format: every shape must be a JSON object in {}",
                shape_name
            )
        );

    let json_stickiness = shape.get("stickiness");

    if json_stickiness.is_none() {
        return None
    }

    let stickiness = json_stickiness
        .unwrap()
        .as_bool()
        .expect(
            &format!
            (
                "Wrong JSON map format: stickiness property must be a boolean in {}",
                shape_name
            )
        );
    
    Some(stickiness)
}



fn parse_json_into_material_index(
    json_shape: &Value,
    shape_name: &str,
    materials_table: &HashMap<String, i32>,
) -> Option<i32> {

    let shape = json_shape
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format: every shape must be a JSON object in {}",
                shape_name
            )
        );

    let json_material = shape.get("visual_material");

    if json_material.is_none() {
        return None
    }

    let material_name = json_material
        .unwrap()
        .as_str()
        .expect(
            &format!
            (
                "Wrong JSON map format: material property must be a string in {}",
                shape_name
            )
        )
        .to_string();

    let material_index = materials_table
        .get(&material_name)
        .expect("wrong material name in a shape")
        .clone();
    
    Some(material_index)
}


fn parse_json_main_actor(
    json: &Value,
    resources: &mut ResourceLoader,
) -> Option<ActorWrapper>
{

    let mut main_actor = None;

    let main_actor_json_obj = json
        .as_object()
        .expect("Wrong JSON map format: main_actor property must be an object");
    
    for (actor_type, actor_value) in main_actor_json_obj {
        match actor_type.as_str() {
            "obstacle_course_free_movement_player" => {

                let actor = parse_obstacle_course_free_movement_player(resources,
                    actor_value,
                );

                main_actor = Some(ActorWrapper::ObstacleCourseFreeMovementPlayer(actor));
            }

            "main_player" => {

                let actor = parse_main_player(resources,
                    actor_value,
                );

                main_actor = Some(ActorWrapper::MainPlayer(actor));
            }
            _ => {panic!("Wrong JSON map format: {} is a wrong main actor type", actor_type)}
        }
    }

    main_actor
}

fn parse_allowed_dynamic_objects_in_bsp_tree(
    obj: &Value
) -> AllowedDynamicObjectsInBspTreeTable {

    let obj = obj
        .as_object()
        .expect("Wrong JSON map format: the value passed to parse_allowed_dynamic_objects_in_bsp_tree must be an object");

    let cubes = obj.get("cubes")
        .expect("Wrong JSON map format: allowed_dynamic_objects_in_bsp_tree must have a cubes property")
        .as_bool()
        .expect("Wrong JSON map format: cubes property of allowed_dynamic_objects_in_bsp_tree must be a boolean");

    let spheres = obj.get("spheres")
        .expect("Wrong JSON map format: allowed_dynamic_objects_in_bsp_tree must have a spheres property")
        .as_bool()
        .expect("Wrong JSON map format: spheres property of allowed_dynamic_objects_in_bsp_tree must be a boolean");

    let sph_cubes = obj.get("sph_cubes")
        .expect("Wrong JSON map format: allowed_dynamic_objects_in_bsp_tree must have a sph_cubes property")
        .as_bool()
        .expect("Wrong JSON map format: sph_cubes property of allowed_dynamic_objects_in_bsp_tree must be a boolean");

    let s_cubes = obj.get("stickiness_cubes")
        .expect("Wrong JSON map format: allowed_dynamic_objects_in_bsp_tree must have a stickiness_cubes property")
        .as_bool()
        .expect("Wrong JSON map format: stickiness_cubes property of allowed_dynamic_objects_in_bsp_tree must be a boolean");

    let s_spheres = obj.get("stickiness_spheres")
        .expect("Wrong JSON map format: allowed_dynamic_objects_in_bsp_tree must have a stickiness_spheres property")
        .as_bool()
        .expect("Wrong JSON map format: stickiness_spheres property of allowed_dynamic_objects_in_bsp_tree must be a boolean");

    let s_sph_cubes = obj.get("stickiness_sph_cubes")
        .expect("Wrong JSON map format: allowed_dynamic_objects_in_bsp_tree must have a stickiness_sph_cubes property")
        .as_bool()
        .expect("Wrong JSON map format: stickiness_sph_cubes property of allowed_dynamic_objects_in_bsp_tree must be a boolean");

    let neg_cubes = obj.get("negative_cubes")
        .expect("Wrong JSON map format: allowed_dynamic_objects_in_bsp_tree must have a negative_cubes property")
        .as_bool()
        .expect("Wrong JSON map format: negative_cubes property of allowed_dynamic_objects_in_bsp_tree must be a boolean");

    let neg_spheres = obj.get("negative_spheres")
        .expect("Wrong JSON map format: allowed_dynamic_objects_in_bsp_tree must have a negative_spheres property")
        .as_bool()
        .expect("Wrong JSON map format: negative_spheres property of allowed_dynamic_objects_in_bsp_tree must be a boolean");

    let neg_sph_cubes = obj.get("negative_sph_cubes")
        .expect("Wrong JSON map format: allowed_dynamic_objects_in_bsp_tree must have a negative_sph_cubes property")
        .as_bool()
        .expect("Wrong JSON map format: negative_sph_cubes property of allowed_dynamic_objects_in_bsp_tree must be a boolean");

    let s_neg_cubes = obj.get("sticiness_negative_cubes")
        .expect("Wrong JSON map format: allowed_dynamic_objects_in_bsp_tree must have a stickiness_negative_cubes property")
        .as_bool()
        .expect("Wrong JSON map format: stickiness_negative_cubes property of allowed_dynamic_objects_in_bsp_tree must be a boolean");

    let s_neg_spheres = obj.get("sticiness_negative_spheres")
        .expect("Wrong JSON map format: allowed_dynamic_objects_in_bsp_tree must have a stickiness_negative_spheres property")
        .as_bool()
        .expect("Wrong JSON map format: stickiness_negative_spheres property of allowed_dynamic_objects_in_bsp_tree must be a boolean");

    let s_neg_sph_cubes = obj.get("sticiness_negative_sph_cubes")
        .expect("Wrong JSON map format: allowed_dynamic_objects_in_bsp_tree must have a stickiness_negative_sph_cubes property")
        .as_bool()
        .expect("Wrong JSON map format: stickiness_negative_sph_cubes property of allowed_dynamic_objects_in_bsp_tree must be a boolean");

    let unbreakable_cubes = obj.get("undestroyable_cubes")
        .expect("Wrong JSON map format: allowed_dynamic_objects_in_bsp_tree must have an undestroyable_cubes property")
        .as_bool()
        .expect("Wrong JSON map format: undestroyable_cubes property of allowed_dynamic_objects_in_bsp_tree must be a boolean");

    let unbreakable_spheres = obj.get("undestroyable_spheres")
        .expect("Wrong JSON map format: allowed_dynamic_objects_in_bsp_tree must have an undestroyable_spheres property")
        .as_bool()
        .expect("Wrong JSON map format: undestroyable_spheres property of allowed_dynamic_objects_in_bsp_tree must be a boolean");

    let unbreakable_sph_cubes = obj.get("undestroyable_sph_cubes")
        .expect("Wrong JSON map format: allowed_dynamic_objects_in_bsp_tree must have an undestroyable_sph_cubes property")
        .as_bool()
        .expect("Wrong JSON map format: undestroyable_sph_cubes property of allowed_dynamic_objects_in_bsp_tree must be a boolean");

    let unbreakable_s_cubes = obj.get("undestroyable_stickiness_cubes")
        .expect("Wrong JSON map format: allowed_dynamic_objects_in_bsp_tree must have an undestroyable_stickiness_cubes property")
        .as_bool()
        .expect("Wrong JSON map format: undestroyable_stickiness_cubes property of allowed_dynamic_objects_in_bsp_tree must be a boolean");

    let unbreakable_s_spheres = obj.get("undestroyable_stickiness_spheres")
        .expect("Wrong JSON map format: allowed_dynamic_objects_in_bsp_tree must have an undestroyable_stickiness_spheres property")
        .as_bool()
        .expect("Wrong JSON map format: undestroyable_stickiness_spheres property of allowed_dynamic_objects_in_bsp_tree must be a boolean");

    let unbreakable_s_sph_cubes = obj.get("undestroyable_stickiness_sph_cubes")
        .expect("Wrong JSON map format: allowed_dynamic_objects_in_bsp_tree must have an undestroyable_stickiness_sph_cubes property")
        .as_bool()
        .expect("Wrong JSON map format: undestroyable_stickiness_sph_cubes property of allowed_dynamic_objects_in_bsp_tree must be a boolean");


    AllowedDynamicObjectsInBspTreeTable {
        cubes,
        spheres,
        sph_cubes,
        s_cubes,
        s_spheres,
        s_sph_cubes,
        neg_cubes,
        neg_spheres,
        neg_sph_cubes,
        s_neg_cubes,
        s_neg_spheres,
        s_neg_sph_cubes,
        unbreakable_cubes,
        unbreakable_spheres,
        unbreakable_sph_cubes,
        unbreakable_s_cubes,
        unbreakable_s_spheres,
        unbreakable_s_sph_cubes,
    }
}

fn parse_main_player(
    resources: &mut ResourceLoader,
    actor_value: &Value,
) -> MainPlayer
{
    let actor_obj = actor_value
        .as_object()
        .expect("Wrong JSON map format: main_player must be a JSON object");

    let red_spawns_val = actor_obj
        .get("red_spawns")
        .expect("Wrong JSON map format: main_player must have a red_spawns property")
        .as_array()
        .expect("Wrong JSON map format: main_player's red_spawns property must be an array");

    let mut red_spawns = Vec::with_capacity(red_spawns_val.len());

    for spawn in red_spawns_val
    {
        let spawn_transform = parse_json_into_transform(spawn, "main_player's red_spawn's spawn");

        red_spawns.push(spawn_transform.get_position());
    }

    let blue_spawns_val = actor_obj
        .get("blue_spawns")
        .expect("Wrong JSON map format: main_player must have a blue_spawns property")
        .as_array()
        .expect("Wrong JSON map format: main_player's blue_spawns property must be an array");

    let mut blue_spawns = Vec::with_capacity(blue_spawns_val.len());

    for spawn in blue_spawns_val
    {
        let spawn_transform = parse_json_into_transform(spawn, "main_player's blue_spawn's spawn");

        blue_spawns.push(spawn_transform.get_position());
    }

    MainPlayer::new(resources, red_spawns, blue_spawns)
}


fn parse_obstacle_course_free_movement_player(
    resources: &mut ResourceLoader,
    actor_value: &Value,
) -> ObstacleCourseFreeMovementPlayer
{

    let spawn_transform_json = actor_value
        .as_object()
        .expect("Wrong JSON map format: obstacle_course_free_movement_player must be a JSON object")
        .get("spawn")
        .expect("Wrong JSON map format: obstacle_course_free_movement_player must have a spawn property");
    
    let spawn_transform = parse_json_into_transform(
        spawn_transform_json,
        "obstacle_course_free_movement_player"
    );

    let with_rotator_tool = actor_value
        .get("with_rotator_tool")
        .expect("Wrong JSON map format: obstacle_course_free_movement_player must have a with_rotator_tool property")
        .as_bool()
        .expect("Wrong JSON map format: with_rotator_tool in obstacle_course_free_movement_player must be a boolean");

    ObstacleCourseFreeMovementPlayer::new(resources,
        spawn_transform.get_position(),
        with_rotator_tool,
    )
}


fn parse_json_actors(
    json: &Value,
    defaults: &DefaultStaticObjectSettings,
    materials_table: &HashMap<String, i32>,
    it_is_2d_3d_example: bool,
    resources: &mut ResourceLoader,
) -> Vec<ActorWrapper>
{

    let mut actors = Vec::new();

    let actors_json_array = json
        .as_array()
        .expect("Wrong JSON map format: actors property must be an array");

    for actor_json_value in actors_json_array {

        let actor_json_object = actor_json_value
            .as_object()
            .expect("Wrong JSON map format: every actor in actors must be a JSON object");

        for (actor_type, actor_value) in actor_json_object {
            match actor_type.as_str() {
                "ctf_session_controller" => {
                    let actor = parse_ctf_session_controller_actor(resources, it_is_2d_3d_example);

                    actors.push(ActorWrapper::SessionController(actor));
                }

                "flag" => {
                    let actor = parse_flag_actor(resources, actor_value);

                    actors.push(ActorWrapper::Flag(actor));
                }

                "flag_base" => {
                    let actor = parse_flag_base_actor(actor_value);

                    actors.push(ActorWrapper::FlagBase(actor));
                }

                "wandering_actor" => {
                    let actor = parse_wandering_actor(actor_value, defaults, materials_table);

                    actors.push(ActorWrapper::WonderingActor(actor));
                }
                "trigger" => {
                    let actor = parse_trigger(actor_value, defaults, materials_table);

                    actors.push(ActorWrapper::Trigger(actor));
                }
                "final_trigger" => {
                    let actor = parse_final_trigger(resources, actor_value, defaults, materials_table);

                    actors.push(ActorWrapper::FinalTrigger(actor));
                }
                "trigger_orb" => {
                    let actor = parse_trigger_orb(resources, actor_value);

                    actors.push(ActorWrapper::TriggerOrb(actor));
                }
                "triggering_wandering_actor" => {
                    let actor = parse_triggering_wandering_actor(actor_value, defaults, materials_table);

                    actors.push(ActorWrapper::TriggeringWanderingActor(actor));
                }
                "triggering_special_lift_actor" => {
                    let actor = parse_triggering_special_lift_actor(actor_value, defaults, materials_table);

                    actors.push(ActorWrapper::TriggeringSpecialLiftActor(actor));
                }
                "droped_rotator_tool" => {
                    let actor = parse_droped_rotator_tool_actor(resources, actor_value);

                    actors.push(ActorWrapper::DropedRotatorTool(actor));
                }
                "new_spawn_area" => {
                    let actor = parse_new_spawn_area_actor(actor_value);

                    actors.push(ActorWrapper::NewSpawnArea(actor));
                }

                _ => {panic!("Wrong JSON map format: {} is a wrong actor type", actor_type)}
            }
        }
    }

    actors
}


fn parse_new_spawn_area_actor(
    actor_value: &Value,
) -> NewSpawnArea {

    let transform = parse_json_into_transform(actor_value, "new_spawn_area_actor");

    let area_size = parse_json_into_size(actor_value, "new_spawn_area_actor");
    
    let new_spawn_position_val = actor_value
        .as_object()
        .expect("Wrong JSON map format: new_spawn_area_actor must be a JSON object")
        .get("new_spawn_position")
        .expect("Wrong JSON map format: new_spawn_area_actor must have a new_spawn_position property");

    let new_spawn_position = parse_json_into_transform(new_spawn_position_val, "new_spawn_area_actor/new_spawn_position").get_position();

    NewSpawnArea::new(
        transform,
        area_size,
        new_spawn_position
    )
}

fn parse_droped_rotator_tool_actor(
    resources: &mut ResourceLoader,
    actor_value: &Value,
) -> DropedRotatorTool {

    let transform = parse_json_into_transform(actor_value, "droped_rotator_tool_actor");

    
    DropedRotatorTool::new(resources,
        transform,
    )
}



fn parse_triggering_special_lift_actor(
    actor_value: &Value,
    defaults: &DefaultStaticObjectSettings,
    materials_table: &HashMap<String, i32>
) -> TriggeringSpecialLiftActor {

    let actor_object = actor_value
        .as_object()
        .expect("Wrong JSON map format: triggering_special_lift_actor must be a JSON object");

    let target = actor_object
        .get("target")
        .expect("Wrong JSON map format: triggering_special_lift_actor must have a target property");

    let transform = parse_json_into_transform(actor_value, "triggering_special_lift_actor");

    let second_target = parse_json_into_transform(target, "triggering_special_lift_actor, target");

    let travel_time = actor_object
        .get("travel_time")
        .expect("Wrong JSON map format: triggering_special_lift_actor must have a travel_time property")
        .as_f64()
        .expect("Wrong JSON map format: travel_time in triggering_special_lift_actor must be a float number")
        as f32;
    
    let stop_time = actor_object
        .get("stop_time")
        .expect("Wrong JSON map format: triggering_special_lift_actor must have a stop_time property")
        .as_f64()
        .expect("Wrong JSON map format: stop_time in triggering_special_lift_actor must be a float number")
        as f32;
    
    let lift_door_open_time = actor_object
        .get("lift_door_open_time")
        .expect("Wrong JSON map format: triggering_special_lift_actor must have a lift_door_open_time property")
        .as_f64()
        .expect("Wrong JSON map format: lift_door_open_time in triggering_special_lift_actor must be a float number")
        as f32;
    
    let lift_door_index = actor_object
        .get("lift_door_index")
        .expect("Wrong JSON map format: triggering_special_lift_actor must have a lift_door_index property")
        .as_u64()
        .expect("Wrong JSON map format: lift_door_index in triggering_special_lift_actor must be an unsigned integer")
        as usize;

    let movement_type = actor_object
        .get("movement_type")
        .expect("Wrong JSON map format: triggering_special_lift_actor must have a movement_type property")
        .as_str()
        .expect("Wrong JSON map format: movement_type in triggering_special_lift_actor must be a string");

    let movement_type = {
        match movement_type {
            "linear" => {
                WanderingActorMovementType::Linear
            },
            "nonlinear" => {
                WanderingActorMovementType::NonLinear
            },
            _ => {
                panic!("Wrong JSON map format: {} is not an allowed movement_type of a wandering actor", movement_type);
            }
        }
    };

    let static_objects_value = actor_object
        .get("static_objects")
        .expect("Wrong JSON map format: triggering_special_lift_actor must have a static_objects property");

    let static_objects = parse_json_static_objects(static_objects_value, defaults, materials_table);

    let triggering_by_trigger = actor_object
        .get("triggering_by_trigger")
        .expect("Wrong JSON map format: triggering_special_lift_actor must have a triggering_by_trigger property")
        .as_str()
        .expect("Wrong JSON map format: triggering_by_trigger in triggering_special_lift_actor must be a string")
        .to_string();

    TriggeringSpecialLiftActor::new(
        transform,
        second_target,
        triggering_by_trigger,
        static_objects,
        lift_door_index,
        lift_door_open_time,
        travel_time,
        stop_time,
        movement_type
    )
}


fn parse_trigger_orb(
    resources: &mut ResourceLoader,
    actor_value: &Value
) -> TriggerOrb
{
    let actor_object = actor_value
        .as_object()
        .expect("Wrong JSON map format: trigger_orb must be a JSON object");

    let target_trigger_name = actor_object
        .get("target_trigger_name")
        .expect("Wrong JSON map format: trigger_orb must have a target_trigger_name property")
        .as_str()
        .expect("Wrong JSON map format: trigger_orb's target_trigger_name property must be a string")
        .to_string();

    let transform = parse_json_into_transform(actor_value, "trigger_orb_actor");

    let radius = actor_object
        .get("radius")
        .expect("Wrong JSON map format: trigger_orb must have a radius property")
        .as_f64()
        .expect("Wrong JSON map format: radius in trigger_orb must be a float number")
        as f32;

    let color = {
        let json_value = actor_object.get("color")
            .expect("Wrong JSON map format: trigger_orb has no color property");

        let color = parse_json_color_and_multiplier(json_value);

        Vec3::new(
            color.x,
            color.y,
            color.z,
        )
    };

    TriggerOrb::new(resources,
        transform,
        target_trigger_name,
        color,
        radius
    )
}


fn parse_final_trigger(
    resources: &mut ResourceLoader,
    actor_value: &Value,
    defaults: &DefaultStaticObjectSettings,
    materials_table: &HashMap<String, i32>
) -> FinalTrgger {

    let actor_object = actor_value
        .as_object()
        .expect("Wrong JSON map format: final_trigger must be a JSON object");

    let next_level_name = actor_object
        .get("next_level_name")
        .expect("Wrong JSON map format: final_trigger must have a next_level_name property")
        .as_str()
        .expect("Wrong JSON map format: final_trigger's next_level_name property must be a string")
        .to_string();

    let async_preload = actor_object
        .get("async_preload")
        .expect("Wrong JSON map format: final_trigger must have an async_preload property")
        .as_bool()
        .expect("Wrong JSON map format: final_trigger's async_preload property must be a boolean");

    // Legacy maps may specify this flag. All transitions now poll readiness.
    if let Some(value) = actor_object.get("async_load") {
        value.as_bool().expect("Wrong JSON map format: final_trigger's async_load property must be a boolean");
    }

    let transform = parse_json_into_transform(actor_value, "final_trgger_actor");

    let trigger_area_radius = actor_object
        .get("trigger_area_radius")
        .expect("Wrong JSON map format: final_trigger must have a trigger_area_radius property")
        .as_f64()
        .expect("Wrong JSON map format: trigger_area_radius in final_trigger must be a float number")
        as f32;
    
    let visual_area_radius = actor_object
        .get("visual_area_radius")
        .expect("Wrong JSON map format: final_trigger must have a visual_area_radius property")
        .as_f64()
        .expect("Wrong JSON map format: visual_area_radius in final_trigger must be a float number")
        as f32;
    
    let coloring_area_radius = actor_object
        .get("coloring_area_radius")
        .expect("Wrong JSON map format: final_trigger must have a coloring_area_radius property")
        .as_f64()
        .expect("Wrong JSON map format: coloring_area_radius in final_trigger must be a float number")
        as f32;

    let static_objects_value = actor_object
        .get("static_objects")
        .expect("Wrong JSON map format: final_trigger must have a static_objects property");

    let visual_area_color = {
        let json_value = actor_object.get("visual_area_color")
            .expect("Wrong JSON map format: final_trigger has no visual_area_color property");

        let color = parse_json_color_and_multiplier(json_value);

        Vec3::new(
            color.x,
            color.y,
            color.z,
        )
    };

    let coloring_area_color = {
        let json_value = actor_object.get("coloring_area_color")
            .expect("Wrong JSON map format: final_trigger has no coloring_area_color property");

        let color = parse_json_color_and_multiplier(json_value);

        Vec3::new(
            color.x,
            color.y,
            color.z,
        )
    };

    let static_objects = parse_json_static_objects(static_objects_value, defaults, materials_table);

    FinalTrgger::new(resources,
        transform,
        next_level_name,
        async_preload,
        trigger_area_radius,
        visual_area_radius,
        visual_area_color,
        Vec4::ZERO,
        coloring_area_radius,
        coloring_area_color,
        Vec4::ZERO,
        static_objects
    )
}


fn parse_ctf_session_controller_actor(resources: &mut ResourceLoader, it_is_2d_3d_example: bool) -> SessionController
{
    SessionController::new(resources, it_is_2d_3d_example)
}

fn parse_flag_actor(
    resources: &mut ResourceLoader,
    actor_value: &Value,
)-> Flag
{
    let actor_object = actor_value
        .as_object()
        .expect("Wrong JSON map format: trigger must be a JSON object");

    let transform_of_the_base = parse_json_into_transform(actor_value, "flag_actor");
    
    let flag_owned_by_team_str = actor_object
        .get("team")
        .expect("Wrong JSON map format: flag must have a team property")
        .as_str()
        .expect("Wrong JSON map format: flag's team property must be a string")
        .to_string();

    let flag_owned_by_team = match flag_owned_by_team_str.as_str()
    {
        "blue" => Team::Blue,

        "red" => Team::Red,

        _ => panic!("Wrong JSON map format: flag's team property must be 'red' or 'blue'")
    };
    
    Flag::new(resources, flag_owned_by_team, transform_of_the_base)
}

fn parse_flag_base_actor(
    actor_value: &Value,
)-> FlagBase
{
    let actor_object = actor_value
        .as_object()
        .expect("Wrong JSON map format: trigger must be a JSON object");

    let transform = parse_json_into_transform(actor_value, "flag_base_actor");
    
    let flag_owned_by_team_str = actor_object
        .get("team")
        .expect("Wrong JSON map format: flag_base must have a team property")
        .as_str()
        .expect("Wrong JSON map format: flag_base's team property must be a string")
        .to_string();
    
    let owned_by_team = match flag_owned_by_team_str.as_str()
    {
        "blue" => Team::Blue,

        "red" => Team::Red,

        _ => panic!("Wrong JSON map format: flag_base's team property must be 'red' or 'blue'")
    };

    FlagBase::new(owned_by_team, transform)
}

fn parse_trigger(
    actor_value: &Value,
    defaults: &DefaultStaticObjectSettings,
    materials_table: &HashMap<String, i32>
) -> Trigger {

    let actor_object = actor_value
        .as_object()
        .expect("Wrong JSON map format: trigger must be a JSON object");

    let trigger_name = actor_object
        .get("trigger_name")
        .expect("Wrong JSON map format: trigger must have a trigger_name property")
        .as_str()
        .expect("Wrong JSON map format: trigger's trigger_name property must be a string")
        .to_string();

    let transform = parse_json_into_transform(actor_value, "trigger_actor");

    let trigger_area_radius = actor_object
        .get("trigger_area_radius")
        .expect("Wrong JSON map format: trigger must have a trigger_area_radius property")
        .as_f64()
        .expect("Wrong JSON map format: trigger_area_radius in trigger must be a float number")
        as f32;
    
    let visual_area_radius = actor_object
        .get("visual_area_radius")
        .expect("Wrong JSON map format: trigger must have a visual_area_radius property")
        .as_f64()
        .expect("Wrong JSON map format: visual_area_radius in trigger must be a float number")
        as f32;
    
    let coloring_area_radius = actor_object
        .get("coloring_area_radius")
        .expect("Wrong JSON map format: trigger must have a coloring_area_radius property")
        .as_f64()
        .expect("Wrong JSON map format: coloring_area_radius in trigger must be a float number")
        as f32;

    let static_objects_value = actor_object
        .get("static_objects")
        .expect("Wrong JSON map format: trigger must have a static_objects property");

    let visual_area_color = {
        let json_value = actor_object.get("visual_area_color")
            .expect("Wrong JSON map format: trigger has no visual_area_color property");

        let color = parse_json_color_and_multiplier(json_value);

        Vec3::new(
            color.x,
            color.y,
            color.z,
        )
    };

    let coloring_area_color = {
        let json_value = actor_object.get("coloring_area_color")
            .expect("Wrong JSON map format: trigger has no coloring_area_color property");

        let color = parse_json_color_and_multiplier(json_value);

        Vec3::new(
            color.x,
            color.y,
            color.z,
        )
    };

    let static_objects = parse_json_static_objects(static_objects_value, defaults, materials_table);

    Trigger::new(
        transform,
        trigger_name,
        trigger_area_radius,
        visual_area_radius,
        visual_area_color,
        Vec4::ZERO,
        coloring_area_radius,
        coloring_area_color,
        Vec4::ZERO,
        static_objects
    )
}


fn parse_triggering_wandering_actor(
    actor_value: &Value,
    defaults: &DefaultStaticObjectSettings,
    materials_table: &HashMap<String, i32>
) -> TriggeringWanderingActor {

    let actor_object = actor_value
        .as_object()
        .expect("Wrong JSON map format: triggering_wandering_actor must be a JSON object");

    let target = actor_object
        .get("target")
        .expect("Wrong JSON map format: triggering_wandering_actor must have a target property");

    let transform = parse_json_into_transform(actor_value, "triggering_wandering_actor");

    let second_target = parse_json_into_transform(target, "triggering_wandering_actor, target");

    let travel_time = actor_object
        .get("travel_time")
        .expect("Wrong JSON map format: triggering_wandering_actor must have a travel_time property")
        .as_f64()
        .expect("Wrong JSON map format: travel_time in triggering_wandering_actor must be a float number")
        as f32;
    
    let movement_type = actor_object
        .get("movement_type")
        .expect("Wrong JSON map format: triggering_wandering_actor must have a movement_type property")
        .as_str()
        .expect("Wrong JSON map format: movement_type in triggering_wandering_actor must be a string");

    let movement_type = {
        match movement_type {
            "linear" => {
                WanderingActorMovementType::Linear
            },
            "nonlinear" => {
                WanderingActorMovementType::NonLinear
            },
            _ => {
                panic!("Wrong JSON map format: {} is not an allowed movement_type of a wandering actor", movement_type);
            }
        }
    };

    let static_objects_value = actor_object
        .get("static_objects")
        .expect("Wrong JSON map format: triggering_wandering_actor must have a static_objects property");

    let static_objects = parse_json_static_objects(static_objects_value, defaults, materials_table);


    let triggering_by_trigger = actor_object
        .get("triggering_by_trigger")
        .expect("Wrong JSON map format: triggering_wandering_actor must have a triggering_by_trigger property")
        .as_str()
        .expect("Wrong JSON map format: triggering_by_trigger in triggering_wandering_actor must be a string")
        .to_string();

    let is_one_time_movement = actor_object
        .get("is_one_time_movement")
        .expect("Wrong JSON map format: triggering_wandering_actor must have an is_one_time_movement property")
        .as_bool()
        .expect("Wrong JSON map format: is_one_time_movement in triggering_wandering_actor must be a boolean");

    TriggeringWanderingActor::new(
        transform,
        second_target,
        triggering_by_trigger,
        is_one_time_movement,
        static_objects,
        travel_time,
        movement_type
    )
}


fn parse_wandering_actor(
    actor_value: &Value,
    defaults: &DefaultStaticObjectSettings,
    materials_table: &HashMap<String, i32>
) -> WanderingActor {

    let actor_object = actor_value
        .as_object()
        .expect("Wrong JSON map format: wandering_actor must be a JSON object");

    let target = actor_object
        .get("target")
        .expect("Wrong JSON map format: wandering_actor must have a target property");

    let transform = parse_json_into_transform(actor_value, "wandering_actor");

    let second_target = parse_json_into_transform(target, "wandering_actor, target");

    let travel_time = actor_object
        .get("travel_time")
        .expect("Wrong JSON map format: wandering_actor must have a travel_time property")
        .as_f64()
        .expect("Wrong JSON map format: travel_time in wandering_actor must be a float number")
        as f32;
    
    let movement_type = actor_object
        .get("movement_type")
        .expect("Wrong JSON map format: wandering_actor must have a movement_type property")
        .as_str()
        .expect("Wrong JSON map format: movement_type in wandering_actor must be a string");

    let movement_type = {
        match movement_type {
            "linear" => {
                WanderingActorMovementType::Linear
            },
            "nonlinear" => {
                WanderingActorMovementType::NonLinear
            },
            _ => {
                panic!("Wrong JSON map format: {} is not an allowed movement_type of a wandering actor", movement_type);
            }
        }
    };

    let static_objects_value = actor_object
        .get("static_objects")
        .expect("Wrong JSON map format: wandering_actor must have a static_objects property");

    let static_objects = parse_json_static_objects(static_objects_value, defaults, materials_table);


    WanderingActor::new(
        transform,
        static_objects,
        second_target,
        travel_time,
        movement_type
    )
}
