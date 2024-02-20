use crate::systems::actor::ActorWrapper;
use crate::systems::physics::{
    static_collider::StaticCollider,
    physics_system_data::ShapeType
};

use super::super::transform::Transform;
use super::static_object::{
    StaticObject,
    ObjectMatrial
};
use glam::{Vec4, Vec3};

use wasm_bindgen_futures::JsFuture;
use serde_json::Value;


pub struct DefaultStaticObjectSettings {
    friction: f32,
    bounce_rate: f32,
    material: ObjectMatrial,
    roundness: f32,
    stickiness: f32,
    is_positive: bool,
}


pub struct Level {
    pub level_name: String,
    pub static_objects: Vec<StaticObject>,
    pub spawn_position: Vec4,
}


impl Level {
    pub async fn download_level_from_server() -> (Level, Vec<ActorWrapper>)
    {
        let window = web_sys::window().unwrap();
    
        let target = "http://127.0.0.1:5500/src/assets/maps/map.json";
        
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
    
        output
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

    let spawn_position = {
        let json_spawn_postition = json_level
            .get("spawn_position")
            .expect("Wrong JSON map format. JSON level must have static_objects property");
        
        let transform = parse_json_into_transform(json_spawn_postition, "spawn_position ");
        
        transform.get_position()
    };

    let default_settings = {
        let json_defaults = json_level
            .get("defaults")
            .expect("Wrong JSON map format. JSON level must have defaults property");

        parse_json_defaults(json_defaults)
    };

    let static_objects = {
        let json_static_objects = json_level
            .get("static_objects")
            .expect("Wrong JSON map format. JSON level must have static_objects property");

        parse_json_static_objects(json_static_objects, default_settings)
    };

    let actors = {
        let json_static_objects = json_level
            .get("static_objects")
            .expect("Wrong JSON map format. JSON level must have static_objects property");

        parse_json_actors(json_static_objects)
    };

    let level = Level {
        level_name,
        static_objects,
        spawn_position,
    };

    (level, actors)
}



fn parse_json_defaults(
    json: &Value
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
            .as_f64()
            .expect("Wrong JSON map format. stickiness value must be float number")
            as f32
    };

    let is_positive = {
        json_settings
            .get("is_positive")
            .expect("Wrong JSON map format. JSON defaults must have is_positive property")
            .as_bool()
            .expect("Wrong JSON map format. is_positive value must be boolean")
    };

    let material = {
        json_settings
            .get("material")
            .expect("Wrong JSON map format. JSON defaults must have material property")
            .as_object()
            .expect("Wrong JSON map format. material value must be object")
    };

    let color = {
        material
            .get("color")
            .expect("Wrong JSON map format. JSON defaults must have color property")
            .as_object()
            .expect("Wrong JSON map format. color value must be object")
    };

    let red = {
        color
            .get("red")
            .expect("Wrong JSON map format. JSON defaults must have red property")
            .as_f64()
            .expect("Wrong JSON map format. red value must be float number")
            as f32
    };

    let green = {
        color
            .get("green")
            .expect("Wrong JSON map format. JSON defaults must have green property")
            .as_f64()
            .expect("Wrong JSON map format. green value must be float number")
            as f32
    };

    let blue = {
        color
            .get("blue")
            .expect("Wrong JSON map format. JSON defaults must have blue property")
            .as_f64()
            .expect("Wrong JSON map format. blue value must be float number")
            as f32
    };

    let color = Vec3::new(red, green, blue);

    let material = ObjectMatrial::new(color);

    DefaultStaticObjectSettings {
        friction,
        bounce_rate,
        material,
        roundness,
        stickiness,
        is_positive
    }
}



fn parse_json_static_objects(
    json_map: &Value, defaults: DefaultStaticObjectSettings
) -> Vec<StaticObject>
{
    let mut static_objects: Vec<StaticObject> = Vec::with_capacity(40);

    let array = json_map
        .as_array()
        .expect("Wrong JSON map format. static objects value must be an array");

    let mut spawn_position = Vec4::ZERO;

    for object in array {
        
        let object = object
            .as_object()
            .expect("Wrong JSON map format, all shape must be json objects");


        for (name, shape) in object {
            let static_object = parse_json_specific_shape(
                name, shape, &defaults
            );
            static_objects.push(static_object)
        }
    }

    static_objects
}



fn parse_json_specific_shape(
    shape_name: &String, json_shape: &Value, defaults: &DefaultStaticObjectSettings
) -> StaticObject {
    let shape_name = shape_name.as_str();

    let shape_type = {
        match shape_name {
            "cube" => {
                ShapeType::Cube
            },
            "cube_w_inf" => {
                ShapeType::CubeInfW
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

    let material = parse_json_into_material(
        json_shape, shape_name
    ).unwrap_or(defaults.material);

    let collider = StaticCollider {
        shape_type,
        position,
        size,
        is_positive,
        friction,
        roundness,
        bounce_rate,
        stickiness,
        actors_id: None
    };

    StaticObject {
        collider,
        material,
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
    
    Transform::new_from_vec4(position)
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



fn parse_json_into_stickiness(json_shape: &Value, shape_name: &str) -> Option<f32> {

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
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format, stickiness property is not float number type in {}",
                shape_name
            )
        ) as f32;
    
    Some(stickiness)
}



fn parse_json_into_material(json_shape: &Value, shape_name: &str) -> Option<ObjectMatrial> {

    let shape = json_shape
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format, all shape must be json objects in {}",
                shape_name
            )
        );

    let json_material = shape.get("material");

    if json_material.is_none() {
        return None
    }

    let material = json_material
        .unwrap()
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format, material property is not object type in {}",
                shape_name
            )
        );
    
    let color = material
        .get("color")
        .expect(
            &format!
            (
                "Wrong JSON map format, color property is not exist in material in {}",
                shape_name
            )
        )
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format, color property is not object type in {}",
                shape_name
            )
        );
    
    let red = color
        .get("red")
        .expect(
            &format!
            (
                "Wrong JSON map format, red property in color is not exist in {}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format, red property in color is not float number type in {}",
                shape_name
            )
        ) as f32;
    
    let green = color
        .get("green")
        .expect(
            &format!
            (
                "Wrong JSON map format, green property in color is not exist in {}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format, green property in color is not float number type in {}",
                shape_name
            )
        ) as f32;
    
    let blue = color
        .get("blue")
        .expect(
            &format!
            (
                "Wrong JSON map format, blue property in color is not exist in {}",
                shape_name
            )
        )
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format, blue property in color is not float number type in {}",
                shape_name
            )
        ) as f32;

    let material = ObjectMatrial::new(
        Vec3::new(red, green, blue)
    );
    
    Some(material)
}



fn parse_json_actors(
    json: &Value
) -> Vec<ActorWrapper>
{
    Vec::new()
}