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
use serde_json::Value;


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
                    let json_map = serde_json::from_str(&text.as_string().unwrap()).unwrap();
                    
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

fn parse_json_into_map(json_map: Value) -> Vec<StaticObject> {
    let mut map: Vec<StaticObject> = Vec::with_capacity(40);

    let array = json_map
        .as_array()
        .expect("Wrong JSON map format. Array elements must be objects");

    for object in array {
        
        let object = object
            .as_object()
            .expect("Wrong JSON map format, all shape must be json objects");


        for (name, shape) in object {
            match name.as_str() {
                "cube" => {
                    let shape_name = "cube shape";

                    let transform = parse_json_into_transform(shape, shape_name);

                    let size = parse_json_into_size(shape, shape_name);

                    let is_positive = parse_json_into_is_positive(shape, shape_name);

                    let shape = StaticObject::Cube(transform, size, is_positive);

                    map.push(shape);
                },
                "cube_w_inf" => {
                    let shape_name = "cube_w_inf";

                    let transform = parse_json_into_transform(shape, shape_name);

                    let size = parse_json_into_size(shape, shape_name);

                    let is_positive = parse_json_into_is_positive(shape, shape_name);

                    let shape = StaticObject::CubeInfW(transform, size, is_positive);

                    map.push(shape);
                },
                "sphere" => {
                    let shape_name = "sphere";

                    let transform = parse_json_into_transform(shape, shape_name);

                    let size = parse_json_into_size(shape, shape_name);

                    let is_positive = parse_json_into_is_positive(shape, shape_name);

                    let shape = StaticObject::Sphere(transform, size, is_positive);

                    map.push(shape);
                },
                "sph_cube" => {
                    let shape_name = "sph_cube";

                    let transform = parse_json_into_transform(shape, shape_name);

                    let size = parse_json_into_size(shape, shape_name);

                    let is_positive = parse_json_into_is_positive(shape, shape_name);

                    let shape = StaticObject::SphCube(transform, size, is_positive);

                    map.push(shape);
                },
                _ => {
                    panic!("Wrong JSON map format, unexpected shape {} in map", name)
                }
            }
        }
    }

    map
}


fn parse_json_into_transform(shape: &Value, shape_name: &'static str) -> Transform {

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
    
    Transform::new(x as f32, y as f32, z as f32, w as f32)
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

fn parse_json_into_is_positive(shape: &Value, shape_name: &'static str) -> bool {

    let shape = shape
        .as_object()
        .expect(
            &format!
            (
                "Wrong JSON map format, all shape must be json objects in {}",
                shape_name
            )
        );

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
                "Wrong JSON map format, size property is not boolean type in {}",
                shape_name
            )
        );
    
    is_positive
}

enum Size {
    X(f32),
    XY(Vec2),
    XYZ(Vec3),
    XYZW(Vec4),
}