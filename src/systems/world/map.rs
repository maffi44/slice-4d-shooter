use crate::systems::transform::{self, Position};

use super::super::{
    static_obj::StaticObject,
    transform::Transform,
};
use glam::{
    Vec2,
    Vec3,
    Vec4,
};
use wasm_bindgen_futures::JsFuture;
use rustc_serialize::json::{Json, Array, Object, self};


pub async fn load_map() -> Vec<StaticObject> {

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
                    let json_map = Json::from_str(&text.as_string().unwrap()).unwrap();
                    
                    parse_json_into_map(json_map)
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

fn parse_json_into_map(json_map: Json) -> Vec<StaticObject> {
    let map: Vec<StaticObject> = Vec::with_capacity(40);

    let array = json_map
        .as_array_mut()
        .expect("Wrong JSON map format. Array elements must be objects");

    for object in array {
        
        let object = object
            .as_object_mut()
            .expect("Wrong JSON map format, all shape must be json objects");


        for (name, content) in object {
            match &name {
                "cube" => {
                    let shape_name = "cube shape";

                    let content = content
                        .as_object_mut()
                        .expect("Wrong JSON map format, all shape must be json objects in " + shape_name);

                    let json_transform = content
                        .get_mut("transform")
                        .expect("Wrong JSON map format, transform property is not exist in " + shape_name);

                    let json_size = content
                        .get_mut("size")
                        .expect("Wrong JSON map format, size property is not exist in " + shape_name);

                    let transform = parse_json_into_transform(json_transform, shape_name);

                    let size = parse_json_into_size(json_size, shape_name);

                    let size = match size {
                        Size::XYZW(size) => size,
                        _ => panic!("Wrong JSON map format, shape cube must have 4 dimensions size")
                    };

                    let shape = StaticObject::Cube(transform, size);

                    map.push(shape);
                },
                // "cube_w_inf" => {
                //     let transform = Transform::new_zero();

                //     let shape = StaticObject::Cube((), ());
                // },
                _ => {
                    panic!()
                }
            }
        }
    }

    map
}


fn parse_json_into_transform(json: Json, shape_name: &'static str) -> Transform {

    let json_transform = json
        .as_object_mut()
        .expect("Wrong JSON map format, transform property is not json object in " + shape_name);

    let json_position = json_transform
        .get_mut("position")
        .expect("Wrong JSON map format, position property is not exist in transform in " + shape_name)
        .as_object_mut()
        .expect("Wrong JSON map format, position property is not json object in " + shape_name);

    let x = json_position
        .get_mut("x")
        .expect("Wrong JSON map format, x property is not json object in " + shape_name)
        .as_f64()
        .expect("Wrong JSON map format, value of x property of transform property is not float number type in " + shape_name);

    let y = json_position
        .get_mut("y")
        .expect("Wrong JSON map format, y property is not json object in " + shape_name)
        .as_f64()
        .expect("Wrong JSON map format, value of y property of transform property is not float number type in " + shape_name);

    let z = json_position
        .get_mut("z")
        .expect("Wrong JSON map format, z property is not json object in " + shape_name)
        .as_f64()
        .expect("Wrong JSON map format, value of z property of transform property is not float number type in " + shape_name);

    let w = json_position
        .get_mut("w")
        .expect("Wrong JSON map format, w property is not json object in " + shape_name)
        .as_f64()
        .expect("Wrong JSON map format, value of w property of transform property is not float number type in " + shape_name);
    
    Transform::new(x, y, z, w)
}



fn parse_json_into_size(json: Json, shape_name: &'static str) -> Size {

    let json_size = json
        .as_object_mut()
        .expect("Wrong JSON map format, size property is not json object in " + shape_name);


    let x = json_size
        .get_mut("x")
        .expect("Wrong JSON map format, x property is not json object in " + shape_name)
        .as_f64()
        .expect("Wrong JSON map format, value of x property of size property is not float number type in " + shape_name);

    
    let y;
    let y_obj = json_size
        .get_mut("y");
    
    if let Some(y_json) = y_obj {
        y = y_json
            .as_f64()
            .expect("Wrong JSON map format, value of y property of size property is not float number type in " + shape_name);
    } else {

        return Size::X(x as f32);
    }


    let z;
    let z_obj = json_size
        .get_mut("z");
    
    if let Some(z_json) = z_obj {
        z = z_json
            .as_f64()
            .expect("Wrong JSON map format, value of z property of size property is not float number type in " + shape_name);
    } else {

        return Size::XY( Vec2::new(x as f32, y as f32) );
    }


    let w;
    let w_obj = json_size
        .get_mut("w");
    
    if let Some(w_json) = w_obj {
        w = w_json
            .as_f64()
            .expect("Wrong JSON map format, value of w property of size property is not float number type in " + shape_name);
    } else {
         
        return Size::XYZ( Vec3::new(x as f32, y as f32, z as f32) );
    }


    Size::XYZW( Vec4::new(x as f32, y as f32, z as f32, w as f32) )

}

enum Size {
    X(f32),
    XY(Vec2),
    XYZ(Vec3),
    XYZW(Vec4),
}