use crate::systems::actor::ActorWrapper;

use super::super::{
    transform::Transform,
};
use glam::{Vec4, Vec3};
use super::static_object::StaticObject;

use wasm_bindgen_futures::JsFuture;
use serde_json::Value;


pub struct ObjectMatrial {
    color: Vec3
}

pub struct DefaultMapSettings {
    friction: f32,
    bounce_rate: f32,
    material: ObjectMatrial,
    roundness: f32,
    stickiness: f32,
    is_positive: bool,
}
pub struct Level {
    level_name: String,
    static_objects: Vec<StaticObject>,
    spawn_position: Vec4,
}

impl Level {
    
    pub async fn download_map_from_server_with_translation(
        translation: Vec4
    ) -> (Level, Vec<ActorWrapper>)
    {
        let window = web_sys::window().unwrap();
    
        let target = "http://127.0.0.1:5500/src/assets/maps/map.json";
        
        let promise = window.fetch_with_str(target);
    
        let result = JsFuture::from(promise).await;
    
        let map = match result {
            Ok(val) => {
                let response: web_sys::Response = val.into();
    
                let json_text = JsFuture::from(response.text().unwrap()).await;
                
                match json_text {
                    Ok(text) => {
                        let json_map = serde_json::from_str(&text.as_string().unwrap()).unwrap();
                        
                        parse_json_into_map_with_translation(json_map, translation)
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
    
        map
    }
}


fn parse_json_into_map_with_translation(
    json_map: Value, translation: Vec4
) -> (Vec<StaticObject>, Vec4)
{
    let mut map: Vec<StaticObject> = Vec::with_capacity(40);

    let array = json_map
        .as_array()
        .expect("Wrong JSON map format. Array elements must be objects");

    let mut spawn_position = Vec4::ZERO;

    for object in array {
        
        let object = object
            .as_object()
            .expect("Wrong JSON map format, all shape must be json objects");


        for (name, shape) in object {
            match name.as_str() {
                "cube" => {
                    let shape_name = "cube shape";

                    let transform = parse_json_into_transform(shape, shape_name, translation);

                    let size = parse_json_into_size(shape, shape_name);

                    let is_positive = parse_json_into_is_positive(shape, shape_name);

                    let shape = StaticObject::Cube(transform, size, is_positive);

                    map.push(shape);
                },
                "cube_w_inf" => {
                    let shape_name = "cube_w_inf";

                    let transform = parse_json_into_transform(shape, shape_name, translation);

                    let size = parse_json_into_size(shape, shape_name);

                    let is_positive = parse_json_into_is_positive(shape, shape_name);

                    let shape = StaticObject::CubeInfW(transform, size, is_positive);

                    map.push(shape);
                },
                "sphere" => {
                    let shape_name = "sphere";

                    let transform = parse_json_into_transform(shape, shape_name, translation);

                    let size = parse_json_into_size(shape, shape_name);

                    let is_positive = parse_json_into_is_positive(shape, shape_name);

                    let shape = StaticObject::Sphere(transform, size, is_positive);

                    map.push(shape);
                },
                "sph_cube" => {
                    let shape_name = "sph_cube";

                    let transform = parse_json_into_transform(shape, shape_name, translation);

                    let size = parse_json_into_size(shape, shape_name);

                    let is_positive = parse_json_into_is_positive(shape, shape_name);

                    let shape = StaticObject::SphCube(transform, size, is_positive);

                    map.push(shape);
                },
                "spawn_position" => {
                    let shape_name = "spawn_position";

                    let transform = parse_json_into_transform(shape, shape_name, translation);

                    spawn_position = transform.get_position();
                }
                _ => {
                    panic!("Wrong JSON map format, unexpected shape {} in map", name)
                }
            }
        }
    }

    (map, spawn_position)
}


fn parse_json_into_transform(shape: &Value, shape_name: &'static str, translation: Vec4) -> Transform {

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
    
    Transform::new_from_vec4(position + translation)
}



fn parse_json_into_size(shape: &Value, shape_name: &'static str) -> Vec4 {

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

fn parse_json_into_is_positive(shape: &Value, shape_name: &'static str) -> Option<bool> {

    let shape = shape.as_object();

    if shape.is_none() {return None;}

    let json_is_positive = shape
        .get("is_positive")
        .expect(
            &format!
            (
                "Wrong JSON map format, is_positive property is not exist in {}",
                shape_name
            )
        );

    let is_positive = json_is_positive
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

fn parse_json_into_friction(shape: &Value, shape_name: &'static str) -> f32 {

    let shape = shape
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format, all shape must be json objects in {}",
                shape_name
            )
        );

    let json_friction = shape
        .get("friction")
        .expect(
            &format!
            (
                "Wrong JSON map format, friction property is not exist in {}",
                shape_name
            )
        );

    let friction = json_friction
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format, friction property is not float type in {}",
                shape_name
            )
        );
    
    Some(friction as f32)
}

fn parse_json_into_bounce_rate(shape: &Value, shape_name: &'static str) -> bool {

    let shape = shape
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format, all shape must be json objects in {}",
                shape_name
            )
        );

    let json_bounce_rate = shape
        .get("bounce_rate")
        .expect(
            &format!
            (
                "Wrong JSON map format, bounce_rate property is not exist in {}",
                shape_name
            )
        );

    let bounce_rate = json_bounce_rate
        .as_f64()
        .expect(
            &format!
            (
                "Wrong JSON map format, bounce_rate property is not float type in {}",
                shape_name
            )
        );
    
    bounce_rate as f32
}