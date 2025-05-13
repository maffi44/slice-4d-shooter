use std::{collections::HashMap, fmt::format, fs::File, io::Read};

use crate::{
    actor::{
        mover_w::MoverW, wandering_actor::{
            WanderingActor,
            WanderingActorMovementType,
        }, ActorWrapper
    }, engine::{
        physics::{
            physics_system_data::ShapeType, static_collider::StaticCollider
        }, world::static_object::{
            ObjectMaterial, StaticObject
        }
    }, transform::Transform
};

use client_server_protocol::Team;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
use glam::{Vec4, Vec3};
use serde_json::Value;

use super::static_object::{WFloor, WRoof};



pub struct DefaultStaticObjectSettings {
    friction: f32,
    bounce_rate: f32,
    default_material_index: i32,
    roundness: f32,
    stickiness: bool,
    is_positive: bool,
    undestroyable: bool,
}

#[derive(Clone, Copy)]
pub struct Spawn {
    pub spawn_position: Vec4,
    pub w_level: usize,
}

pub struct EnvirnomentVisualSettings {
    pub sky_box_name: String,
    pub sky_color: Vec4,
    pub fog_color: Vec4,
    pub frenel_color: Vec4,
    pub neon_wireframe_color: Vec4,
    pub sun_color: Vec4,
    pub sun_direction: Vec4,
    pub red_map_color: Vec4,
    pub blue_map_color: Vec4,
}

pub struct Level {
    pub level_name: String,
    pub static_objects: Vec<StaticObject>,
    pub red_spawns: Vec<Spawn>,
    pub blue_spawns: Vec<Spawn>,
    pub all_shapes_stickiness_radius: f32,
    pub w_floor: Option<WFloor>,
    pub w_roof: Option<WRoof>,
    pub visual_materials: Vec<ObjectMaterial>,
    pub blue_players_visual_materials: (i32, i32),
    pub red_players_visual_materials: (i32, i32),
    pub w_cups_visual_materials: i32,
    pub visual_settings_of_environment: EnvirnomentVisualSettings,

    pub w_levels: Vec<f32>,

    pub blue_base_w_level: f32,
    pub red_base_w_level: f32,

    // pub blue_map_color_level: f32,
    // pub red_map_color_level: f32,
    pub red_flag_base: Transform,
    pub blue_flag_base: Transform,
    pub move_w_bonus_spot: Transform,

    pub mover_w_list: Vec<MoverW>,
}

impl Level {
    
    pub fn get_random_spawn_position(&self, team: Team) -> Spawn {
        match team {
            Team::Red =>
            {
                let random_index = {
                    let mut usize_bytes = 0usize.to_be_bytes();
                    getrandom::getrandom(&mut usize_bytes).expect("Can not make random usize in get_random_spawn_position func");
                    
                    usize::from_le_bytes(usize_bytes) % self.red_spawns.len()
                };
        
                self.red_spawns[random_index].clone()
            }

            Team::Blue =>
            {
                let random_index = {
                    let mut usize_bytes = 0usize.to_be_bytes();
                    getrandom::getrandom(&mut usize_bytes).expect("Can not make random usize in get_random_spawn_position func");
                    
                    usize::from_le_bytes(usize_bytes) % self.blue_spawns.len()
                };
        
                self.blue_spawns[random_index].clone()
            }
        }
    }


    pub async fn load_level(level_name: String) -> (Level, Vec<ActorWrapper>)
    {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
        
            let target = format!("http://127.0.0.1:5500/src/assets/maps/{}.json", level_name);
            
            let promise = window.fetch_with_str(target);
        
            let result = JsFuture::from(promise).await;
        
            let output = match result {
                Ok(val) => {
                    let response: web_sys::Response = val.into();
        
                    let json_text = JsFuture::from(response.text().unwrap()).await;
                    
                    match json_text {
                        Ok(text) => {
                            let json_map = serde_json::from_str(&text.as_string().unwrap()).unwrap();
                            
                            parse_json_level(json_map)
                        },
                        Err(val) => {
                            panic!(
                                "ERROR: cannot converting map.json file into text, err is: {:?}",
                                val
                            );
                        },
                    }
                },
                Err(val) => {
                    panic!(
                        "ERROR: the map cannot be loaded, err: {}",
                        val.as_string().unwrap_or("".to_string())
                    );
                }  
            };
        
            return output;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let json_map = {
                let mut file = File::open("./map.json");

                if file.is_err()
                {
                    file = File::open(
                        format!("./src/assets/maps/{}.json", level_name)
                    )
                }
                if file.is_err()
                {
                    file = File::open(
                        format!("/home/maffi/Dream/web-engine4d/src/assets/maps/{}.json", level_name)
                    );
                }

                if file.is_ok()
                {
                    let mut file_content = String::new();
                    match file.unwrap().read_to_string(&mut file_content) {
                        Ok(_) => {
                            serde_json::from_str(&file_content)
                                .expect("Can't parse map.json file")
                        },
                        Err(e) => {
                            println!(
                                "ERROR: the map.json cannot be loaded, err: {}",
                                e.to_string()
                            );
                            serde_json::from_str(include_str!("../../../src/assets/maps/map.json"))
                                .expect("Can't parse map.json file")
                        }
                    }
                }
                else
                {
                    serde_json::from_str(include_str!("../../../src/assets/maps/map.json"))
                        .expect("Can't parse map.json file")
                }
            };

            return parse_json_level(json_map);
        }
    }
}



fn parse_json_level(
    json: Value
) -> (Level, Vec<ActorWrapper>) {
    let json_level = json
    .as_object()
    .expect("Wrong JSON map format. Root json level value must be object");

    let level_name: String = {
        json_level
            .get("level_name")
            .expect("Wrong JSON map format. JSON level must have level_name property")
            .as_str()
            .expect("Wrong JSON map format. level name value must be string")
            .into()
    };

    let all_shapes_stickiness_radius: f32 = {
        json_level
            .get("all_shapes_stickiness_radius")
            .expect("Wrong JSON map format. JSON level must have all_shapes_stickiness_radius property")
            .as_f64()
            .expect("Wrong JSON map format. all_shapes_stickiness_radius value must be float number")
            as f32
    };

    let red_spawns = {
        let spawns_array = json_level
            .get("red_spawns")
            .expect("Wrong JSON map format. JSON level must have red_spawns property")
            .as_array()
            .expect("red_spawns is not an array");

        let mut spawns = Vec::new();

        for value in spawns_array {
            let transform = parse_json_into_transform(value, "spawn_position");
            let w_level = {
                
                let obj = value
                    .as_object()
                    .expect("Wrong JSON map format, type of spawn is not an object");
            
                obj
                    .get("w_level")
                    .expect("Wrong JSON map format, spawn have not w_level property")
                    .as_u64()
                    .expect("Wrong JSON map format, w_level of spawn is not number")
                    as usize
            };

            let spawn = Spawn {
                spawn_position: transform.get_position(),
                w_level,
            };

            spawns.push(spawn);
        }

        spawns  
    };

    let blue_spawns = {
        let spawns_array = json_level
            .get("blue_spawns")
            .expect("Wrong JSON map format. JSON level must have blue_spawns property")
            .as_array()
            .expect("blue_spawns is not an array");

        let mut spawns = Vec::new();

        for value in spawns_array {
            let transform = parse_json_into_transform(value, "spawn_position");
            let w_level = {
                
                let obj = value
                    .as_object()
                    .expect("Wrong JSON map format, type of spawn is not an object");
            
                obj
                    .get("w_level")
                    .expect("Wrong JSON map format, spawn have not w_level property")
                    .as_u64()
                    .expect("Wrong JSON map format, w_level of spawn is not number")
                    as usize
            };

            let spawn = Spawn {
                spawn_position: transform.get_position(),
                w_level,
            };

            spawns.push(spawn);
        }

        spawns  
    };


    let (visual_materials, materials_table) = {
        let json_visual_materials = json_level
            .get("visual_materials")
            .expect("Wrong JSON map format. JSON level must have visual_materials property");

        parse_visual_materials(json_visual_materials)
    };

    let default_settings = {
        let json_defaults = json_level
            .get("defaults")
            .expect("Wrong JSON map format. JSON level must have defaults property");

        parse_json_defaults(json_defaults, &materials_table)
    };

    let red_players_visual_materials = {
        let json_players_visual_materials = json_level
            .get("red_players_visual_materials")
            .expect("Wrong JSON map format. JSON level must have red_players_visual_materials property");

        parse_players_visual_materials(json_players_visual_materials, &materials_table)
    };

    let blue_players_visual_materials = {
        let json_players_visual_materials = json_level
            .get("blue_players_visual_materials")
            .expect("Wrong JSON map format. JSON level must have blue_players_visual_materials property");

        parse_players_visual_materials(json_players_visual_materials, &materials_table)
    };

    let w_cups_visual_materials = {
        let json_players_visual_materials = json_level
            .get("w_cups_visual_materials")
            .expect("Wrong JSON map format. JSON level must have w_cups_visual_materials property");

        parse_w_cups_visual_materials(json_players_visual_materials, &materials_table)
    };

    let (w_floor, w_roof) = {
        let json_w_cups = json_level
            .get("w_cups")
            .expect("Wrong JSON map format. JSON level must have w_cups property");

        parse_w_cups(json_w_cups)
    };

    let static_objects = {
        let json_static_objects = json_level
            .get("static_objects")
            .expect("Wrong JSON map format. JSON level must have static_objects property");

        parse_json_static_objects(json_static_objects, &default_settings, &materials_table)
    };

    let actors = {
        let json_actors = json_level
            .get("actors")
            .expect("Wrong JSON map format. JSON level must have static_objects property");

        parse_json_actors(json_actors, &default_settings, &materials_table)
    };

    let w_levels = {
        let mut w_levels = Vec::new();

        let array = json_level
            .get("w_levels")
            .expect("Wrong JSON map format. JSON level must have w_levels property")
            .as_array()
            .expect("Wrong JSON map format. JSON's w_levels property must be an array");

        for value in array {
            let w_level = value
                .as_f64()
                .expect("Wrong JSON map format. JSON's w_levels array's value is not a number")
                as f32;
            
            w_levels.push(w_level);
        }

        w_levels
    };

    let visual_settings_of_environment = {
        let json_visual_settings_of_environment = json_level
            .get("visual_settings_of_environment")
            .expect("Wrong JSON map format. JSON level must have visual_settings_of_environment property");

        parse_json_visual_settings_of_environment(json_visual_settings_of_environment)
    };

    // let blue_map_color_level = {
    //     json_level
    //         .get("blue_map_color_level")
    //         .expect("Wrong JSON map format. JSON level must have blue_map_color_level property")
    //         .as_f64()
    //         .expect("Wrong JSON map format. JSON's blue_map_color_level property must be an number")
    //         as f32
    // };
    
    // let red_map_color_level = {
    //     json_level
    //         .get("red_map_color_level")
    //         .expect("Wrong JSON map format. JSON level must have red_map_color_level property")
    //         .as_f64()
    //         .expect("Wrong JSON map format. JSON's red_map_color_level property must be an number")
    //         as f32
    // };
    
    let red_flag_base = {
        let red_flag_base_json = json_level
            .get("red_flag_base")
            .expect("Wrong JSON map format. JSON level must have red_flag_base property");

        parse_json_into_transform(red_flag_base_json, "red_flag_base")
    };
    
    let blue_flag_base = {
        let blue_flag_base_json = json_level
            .get("blue_flag_base")
            .expect("Wrong JSON map format. JSON level must have blue_flag_base property");

        parse_json_into_transform(blue_flag_base_json, "blue_flag_base")
    };
    
    let move_w_bonus_spot = {
        let move_w_bonus_spot_json = json_level
            .get("move_w_bonus_spot")
            .expect("Wrong JSON map format. JSON level must have move_w_bonus_spot property");

        parse_json_into_transform(move_w_bonus_spot_json, "move_w_bonus_spot")
    };

    let blue_base_w_level = {
        json_level
            .get("blue_base_w_level")
            .expect("Wrong JSON map format. JSON level must have blue_base_w_level property")
            .as_f64()
            .expect("Wrong JSON map format. JSON's blue_base_w_level property must be an number")
            as f32
    };
    let red_base_w_level = {
        json_level
            .get("red_base_w_level")
            .expect("Wrong JSON map format. JSON level must have red_base_w_level property")
            .as_f64()
            .expect("Wrong JSON map format. JSON's red_base_w_level property must be an number")
            as f32
    };

    let mover_w_list = {
        let array = json_level
            .get("mover_w_list")
            .expect("Wrong JSON map format. JSON level must have mover_w_list property")
            .as_array()
            .expect("Wrong JSON map format. JSON's mover_w_list property must be an array");

        let mut list = Vec::new();

        for json_obj in array
        {
            let mover_w = parse_mover_w(json_obj, &w_levels);

            list.push(mover_w);
        };

        list
    };

    let level = Level {
        blue_base_w_level,
        red_base_w_level,
        level_name,
        static_objects,
        red_spawns,
        blue_spawns,
        all_shapes_stickiness_radius,
        w_floor,
        w_roof,
        visual_materials,
        blue_players_visual_materials,
        red_players_visual_materials,
        w_cups_visual_materials,
        w_levels,
        visual_settings_of_environment,
        red_flag_base,
        blue_flag_base,
        // red_map_color_level,
        // blue_map_color_level,
        move_w_bonus_spot,
        mover_w_list,
    };

    (level, actors)
}


fn parse_mover_w(json_obj: &Value, w_levels: &Vec<f32>) -> MoverW
{
    let transform = parse_json_into_transform(json_obj, "mover_w");

    let direction = {
        json_obj.get("direction")
            .expect("Wrong JSON map format. mover_w have not an direction property")
            .as_f64()
            .expect("Wrong JSON map format. mover_w have's direction property is not a number")
            as f32
    };

    MoverW::new(transform.get_position(), direction, w_levels)
}


fn parse_json_visual_settings_of_environment(
    json: &Value,
) -> EnvirnomentVisualSettings {
    let obj = json
        .as_object()
        .expect("Wrong JSON map format. Root of visual_settings_of_environment property must be an object");

    let sky_box_name = {
        obj.get("sky_box")
            .expect("Wrong JSON map format. visual_settings_of_environment have not an sky_box property")
            .as_str()
            .expect("Wrong JSON map format. visual_settings_of_environment's sky_box property is not a string")
            .to_string()
    };

    let sky_color = {
        let json_value = obj.get("sky_color")
            .expect("Wrong JSON map format. visual_settings_of_environment have not an sky_color property");

        parse_json_color_and_multiplier(json_value)
    };

    let fog_color = {
        let json_value = obj.get("fog_color")
            .expect("Wrong JSON map format. visual_settings_of_environment have not an fog_color property");

        parse_json_color_and_multiplier(json_value)
    };

    let frenel_color = {
        let json_value = obj.get("frenel_color")
            .expect("Wrong JSON map format. visual_settings_of_environment have not an frenel_color property");

        parse_json_color_and_multiplier(json_value)
    };

    let neon_wireframe_color = {
        let json_value = obj.get("neon_wireframe_color")
            .expect("Wrong JSON map format. visual_settings_of_environment have not an neon_wireframe_color property");

        parse_json_color_and_multiplier(json_value)
    };

    let sun_color = {
        let json_value = obj.get("sun_color")
            .expect("Wrong JSON map format. visual_settings_of_environment have not an sun_color property");

        parse_json_color_and_multiplier(json_value)
    };

    let sun_direction = {
        let json_value = obj.get("sun_direction")
            .expect("Wrong JSON map format. visual_settings_of_environment have not an sun_direction property");

        let x = json_value
            .get("x")
            .expect("Wrong JSON map format, x property is not exist in sun_direction")
            .as_f64()
            .expect("Wrong JSON map format, value of x property of sun_direction is not float number");
    
        let y = json_value
            .get("y")
            .expect("Wrong JSON map format, y property is not exist in sun_direction")
            .as_f64()
            .expect("Wrong JSON map format, value of y property of sun_direction is not float number");
    
        let z = json_value
            .get("z")
            .expect("Wrong JSON map format, z property is not exist in sun_direction")
            .as_f64()
            .expect("Wrong JSON map format, value of z property of sun_direction is not float number");
    
        let w = json_value
            .get("w")
            .expect("Wrong JSON map format, w property is not exist in sun_direction")
            .as_f64()
            .expect("Wrong JSON map format, value of w property of sun_direction is not float number");
        
        Vec4::new(x as f32, y as f32, z as f32, w as f32)
    };

    let red_map_color = {
        let json_value = obj.get("red_map_color")
            .expect("Wrong JSON map format. visual_settings_of_environment have not an red_map_color property");

        parse_json_color_and_multiplier(json_value)
    };

    let blue_map_color = {
        let json_value = obj.get("blue_map_color")
            .expect("Wrong JSON map format. visual_settings_of_environment have not an blue_map_color property");

        parse_json_color_and_multiplier(json_value)
    };

    EnvirnomentVisualSettings {
        sky_box_name,
        sky_color,
        sun_color,
        sun_direction,
        fog_color,
        frenel_color,
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
        .expect("Wrong JSON map format. Root of some of visual_settings_of_environment propertys must be an object");

    let red = {
        obj
            .get("red")
            .expect("Wrong JSON map format. color must have red property")
            .as_f64()
            .expect("Wrong JSON map format. red value must be float number")
            as f32
    };

    let green = {
        obj
            .get("green")
            .expect("Wrong JSON map format. color must have green property")
            .as_f64()
            .expect("Wrong JSON map format. green value must be float number")
            as f32
    };

    let blue = {
        obj
            .get("blue")
            .expect("Wrong JSON map format. color must have blue property")
            .as_f64()
            .expect("Wrong JSON map format. blue value must be float number")
            as f32
    };

    let mult = {
        obj
            .get("multiplier")
            .expect("Wrong JSON map format. color must have multiplier property")
            .as_f64()
            .expect("Wrong JSON map format. multiplier value must be float number")
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
        .expect("Wrong JSON map format. Root of players_visual_materials property must be an object");

    let inner_material_name = obj
        .get("inner")
        .expect("players_visual_materials have not an inner property")
        .as_str()
        .expect("inner property in players_visual_materials is not a string")
        .to_string();

    let inner_material_index = materials_table
        .get(&inner_material_name)
        .expect("inner material in players_visual_materials not exist")
        .clone();

    let outer_material_name = obj
        .get("outer")
        .expect("players_visual_materials have not an outer property")
        .as_str()
        .expect("outer property in players_visual_materials is not a string")
        .to_string();

    let outer_material_index = materials_table
        .get(&outer_material_name)
        .expect("outer material in players_visual_materials not exist")
        .clone();

    (inner_material_index, outer_material_index)
}


fn parse_w_cups_visual_materials(
    json: &Value,
    materials_table: &HashMap<String, i32>
) -> i32 {
    let material_name = json
        .as_str()
        .expect("Wrong JSON map format. Value of w_cups_visual_materials property must be an string")
        .to_string();

    let material_index = materials_table
        .get(&material_name)
        .expect("material in w_cups_visual_materials not exist")
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
        .expect("Wrong JSON map format. Root of visual_materials property must be array");

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
        .expect("Wrong JSON map format. Material in visual_materials must be an object");

    let name = obj.get("name")
        .expect("Visual material in Visual materials have not name property")
        .as_str()
        .expect("Property name in Visual Material is not an string")
        .to_string();

        let color = obj
                .get("color")
                .expect("Wrong JSON map format. material must have color property")
                .as_object()
                .expect("Wrong JSON map format. color value must be object");
    
        let red = {
            color
                .get("red")
                .expect("Wrong JSON map format. color must have red property")
                .as_f64()
                .expect("Wrong JSON map format. red value must be float number")
                as f32
        };
    
        let green = {
            color
                .get("green")
                .expect("Wrong JSON map format. color must have green property")
                .as_f64()
                .expect("Wrong JSON map format. green value must be float number")
                as f32
        };
    
        let blue = {
            color
                .get("blue")
                .expect("Wrong JSON map format. color must have blue property")
                .as_f64()
                .expect("Wrong JSON map format. blue value must be float number")
                as f32
        };
    
        let color = Vec3::new(red, green, blue);

        let roughness = {
            obj
                .get("roughness")
                .expect("Wrong JSON map format. material must have roughness property")
                .as_f64()
                .expect("Wrong JSON map format. roughness value must be number")
                as f32
        };
    
        let material = ObjectMaterial::new(color, roughness);

        (material, name)
}


fn parse_w_cups(
    json: &Value
) -> (Option<WFloor>, Option<WRoof>) {
    let obj = json
        .as_object()
        .expect("Wrong JSON map format. Root of w_cups property must be an object");

    let w_floor = {
        if let Some(val) = obj.get("w_floor") {
            let val = val
                .as_f64()
                .expect("Wrong JSON map format. Value of w_floor property in w_cups object must be a number")
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
                .expect("Wrong JSON map format. Value of w_roof property in w_cups object must be a number")
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
        .expect("Wrong JSON map format. Root json level value must be object");

    let friction = {
        json_settings
            .get("friction")
            .expect("Wrong JSON map format. JSON defaults must have friction property")
            .as_f64()
            .expect("Wrong JSON map format. friction value must be float number")
            as f32
    };

    let bounce_rate = {
        json_settings
            .get("bounce_rate")
            .expect("Wrong JSON map format. JSON defaults must have bounce_rate property")
            .as_f64()
            .expect("Wrong JSON map format. bounce_rate value must be float number")
            as f32
    };

    let roundness = {
        json_settings
            .get("roundness")
            .expect("Wrong JSON map format. JSON defaults must have roundness property")
            .as_f64()
            .expect("Wrong JSON map format. roundness value must be float number")
            as f32
    };

    let stickiness = {
        json_settings
            .get("stickiness")
            .expect("Wrong JSON map format. JSON defaults must have stickiness property")
            .as_bool()
            .expect("Wrong JSON map format. stickiness value must be boolean type")
    };

    let is_positive = {
        json_settings
            .get("is_positive")
            .expect("Wrong JSON map format. JSON defaults must have is_positive property")
            .as_bool()
            .expect("Wrong JSON map format. is_positive value must be boolean")
    };

    let visual_material_name = json_settings
        .get("visual_material")
        .expect("Wrong JSON map format. JSON defaults must have visual_material property")
        .as_str()
        .expect("Wrong JSON map format. visual_material value must be object")
        .to_string();
    
    let default_material_index = materials_table
        .get(&visual_material_name)
        .expect("default visual material in defaults have wrong material name")
        .clone();

    let undestroyable = {
        json_settings
            .get("undestroyable")
            .expect("Wrong JSON map format. JSON defaults must have undestroyable property")
            .as_bool()
            .expect("Wrong JSON map format. undestroyable value must be boolean")
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
        .expect("Wrong JSON map format. static objects value must be an array");

    for object in array {
        
        let object = object
            .as_object()
            .expect("Wrong JSON map format, all shape must be json objects");


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
                panic!("cube_w_inf shapes are temporarily not supported")
                // ShapeType::CubeInfW
            },
            "sphere" => {
                ShapeType::Sphere
            },
            "sph_cube" => {
                ShapeType::SphCube
            },
            _ => {
                panic!("Wrong JSON map format, unexpected shape {} in map", shape_name)
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
                "Wrong JSON map format, all shape must be json objects in {}",
                shape_name
            )
        );

    let json_transform = shape
        .get("transform")
        .expect(
            &format!
            (
                "Wrong JSON map format, transform property is not exist in {}",
                shape_name
            )
        );

    let json_transform = json_transform
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format, transform property is not json object in {}",
                shape_name
            )
        );

    let json_position = json_transform
        .get("position")
        .expect(
            &format!
            (
                "Wrong JSON map format, position property is not exist in transform in {}",
                shape_name
            )
        )
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format, position property is not json object in {}",
                shape_name
            )
        );
            

    let x = json_position
        .get("x")
        .expect(
            &format!
            (
                "Wrong JSON map format, x property is not exist in transform in {}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format, value of x property of transform property is not float number type in {}",
                shape_name
            )
        );

    let y = json_position
        .get("y")
        .expect(
            &format!
            (
                "Wrong JSON map format, y property is not exist in transform in{}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format, value of y property of transform property is not float number type in {}",
                shape_name
            )
        );

    let z = json_position
        .get("z")
        .expect(
            &format!
            (
                "Wrong JSON map format, z property is not exist in transform in {}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format, value of z property of transform property is not float number type in {}",
                shape_name
            )
        );

    let w = json_position
        .get("w")
        .expect(
            &format!
            (
                "Wrong JSON map format, w property is not exist in transform in {}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format, value of w property of transform property is not float number type in {}",
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
            "Wrong JSON map format, scale property is not exist in transform in {}",
            shape_name
        )
    )
    .as_object()
    .expect(
        &format!
        (
            "Wrong JSON map format, scale property is not json object in {}",
            shape_name
        )
    );
            
    let x = json_scale
        .get("x")
        .expect(
            &format!
            (
                "Wrong JSON map format, x property is not exist in transform in {}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format, value of x property of transform property is not float number type in {}",
                shape_name
            )
        );

    let y = json_scale
        .get("y")
        .expect(
            &format!
            (
                "Wrong JSON map format, y property is not exist in transform in{}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format, value of y property of transform property is not float number type in {}",
                shape_name
            )
        );

    let z = json_scale
        .get("z")
        .expect(
            &format!
            (
                "Wrong JSON map format, z property is not exist in transform in {}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format, value of z property of transform property is not float number type in {}",
                shape_name
            )
        );

    let w = json_scale
        .get("w")
        .expect(
            &format!
            (
                "Wrong JSON map format, w property is not exist in transform in {}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format, value of w property of transform property is not float number type in {}",
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
                "Wrong JSON map format, all shape must be json objects in {}",
                shape_name
            )
        );

    let json_size = shape
        .get("size")
        .expect(
            &format!
            (
                "Wrong JSON map format, size property is not exist in {}",
                shape_name
            )
        );

    let json_size = json_size
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format, size property is not json object in {}",
                shape_name
            )
        );

    let x = json_size
        .get("x")
        .expect(
            &format!
            (
                "Wrong JSON map format, x property is not exist in size in {}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format, value of x property of size property is not float number type in {}",
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
                    "Wrong JSON map format, value of y property of size property is not float number type in {}",
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
                    "Wrong JSON map format, value of z property of size property is not float number type in {}",
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
                    "Wrong JSON map format, value of w property of size property is not float number type in {}",
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
                "Wrong JSON map format, all shape must be json objects in {}",
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
                "Wrong JSON map format, is_positive property is not boolean type in {}",
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
                "Wrong JSON map format, all shape must be json objects in {}",
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
                "Wrong JSON map format, undestroyable property is not boolean type in {}",
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
                "Wrong JSON map format, all shape must be json objects in {}",
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
                "Wrong JSON map format, friction property is not float number type in {}",
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
                "Wrong JSON map format, all shape must be json objects in {}",
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
                "Wrong JSON map format, bounce_rate property is not float number type in {}",
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
                "Wrong JSON map format, all shape must be json float number in {}",
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
                "Wrong JSON map format, roundness property is not float number type in {}",
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
                "Wrong JSON map format, all shape must be json objects in {}",
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
                "Wrong JSON map format, stickiness property is not boolean type in {}",
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
                "Wrong JSON map format, all shape must be json objects in {}",
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
                "Wrong JSON map format, material property is not string type in {}",
                shape_name
            )
        )
        .to_string();

    let material_index = materials_table
        .get(&material_name)
        .expect("Wrong material name in shape")
        .clone();
    
    Some(material_index)
}



fn parse_json_actors(
    json: &Value,
    defaults: &DefaultStaticObjectSettings,
    materials_table: &HashMap<String, i32>,
) -> Vec<ActorWrapper>
{

    let mut actors = Vec::new();

    let actors_json_array = json
        .as_array()
        .expect("Wrong JSON map format, actors property must be array");

    for actor_json_value in actors_json_array {

        let actor_json_object = actor_json_value
            .as_object()
            .expect("Wrong JSON map format, any actor in actors must be a json object");

        for (actor_type, actor_value) in actor_json_object {
            match actor_type.as_str() {
                "wandering_actor" => {

                    let actor = parse_wandering_actor(actor_value, defaults, materials_table);

                    actors.push(ActorWrapper::WonderingActor(actor));
                }
                _ => {panic!("Wrong JSON map format, {} it is worng actor type", actor_type)}
            }
        }
    }

    actors
}


fn parse_wandering_actor(
    actor_value: &Value,
    defaults: &DefaultStaticObjectSettings,
    materials_table: &HashMap<String, i32>
) -> WanderingActor {

    let actor_object = actor_value
        .as_object()
        .expect("Wrong JSON map format, wandering_actor must be an json object");

    let target = actor_object
        .get("target")
        .expect("Wrong JSON map format, wandering_actor must have target property");

    let transform = parse_json_into_transform(actor_value, "wandering_actor");

    let second_target = parse_json_into_transform(target, "wandering_actor, target");

    let travel_time = actor_object
        .get("travel_time")
        .expect("Wrong JSON map format, wandering_actor must have travel_time property")
        .as_f64()
        .expect("Wrong JSON map format, travel_time in wandering_actor must be float number")
        as f32;
    
    let movement_type = actor_object
        .get("movement_type")
        .expect("Wrong JSON map format, wandering_actor must have movement_type property")
        .as_str()
        .expect("Wrong JSON map format, movement_type in wandering_actor must be string value");

    let movement_type = {
        match movement_type {
            "linear" => {
                WanderingActorMovementType::Linear
            },
            "nonlinear" => {
                WanderingActorMovementType::NonLinear
            },
            _ => {
                panic!("Wrong JSON map format, {} it is not allowed movement_type in wandering actor", movement_type);
            }
        }
    };

    let static_objects_value = actor_object
        .get("static_objects")
        .expect("Wrong JSON map format, wandering_actor must have static_objects property");

    let static_objects = parse_json_static_objects(static_objects_value, defaults, materials_table);


    WanderingActor::new(
        transform,
        static_objects,
        second_target,
        travel_time,
        movement_type
    )
}
