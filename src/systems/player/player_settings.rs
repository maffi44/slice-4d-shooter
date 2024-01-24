use wasm_bindgen_futures::JsFuture;
use serde_json::Value;

#[derive(Clone, Copy)]
pub struct PlayerSettings {

    pub collider_radius: f32,

    pub max_speed: f32,
    pub max_accel: f32,

    pub air_speed_mult: f32,

    pub jump_y_speed: f32,
    pub jump_w_speed: f32,

    pub jetpak_w_speed: f32,

    pub gravity_y_speed: f32,
    pub gravity_w_speed: f32,

    pub friction_on_ground: f32,
    pub friction_on_air: f32,
}

impl PlayerSettings {
    pub async fn load_player_settings() -> Self {

        let window = web_sys::window().unwrap();

        let target = "http://127.0.0.1:5500/src/assets/maps/settings.json";
        
        let promise = window.fetch_with_str(target);
    
        let result = JsFuture::from(promise).await;
    
        let player_settings = match result {
            Ok(val) => {
                let response: web_sys::Response = val.into();
    
                let json_text = JsFuture::from(response.text().unwrap()).await;
                
                match json_text {
                    Ok(text) => {
                        let json_settings = serde_json::from_str(&text.as_string().unwrap()).unwrap();
                        
                        parse_json_into_settings(json_settings)
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
    
        player_settings
    }
}

fn parse_json_into_settings(json_settigs: Value) -> PlayerSettings {

    let mut settings = PlayerSettings {
        collider_radius: f32::NAN,
        max_speed: f32::NAN,
        max_accel: f32::NAN,
        air_speed_mult: f32::NAN,
        jump_y_speed: f32::NAN,
        jump_w_speed: f32::NAN,
        jetpak_w_speed: f32::NAN,
        gravity_y_speed: f32::NAN,
        gravity_w_speed: f32::NAN,
        friction_on_ground: f32::NAN,
        friction_on_air: f32::NAN,
    };

    let object = json_settigs
        .as_object()
        .expect("Wrong JSON settings format");

        for (property, value) in object {
            match property.as_str() {
                "player_sphere_radius" => {
                    settings.collider_radius = {
                        value.as_f64().expect("Wrong JSON settings format") as f32
                    }
                },
                "player_max_speed" => {
                    settings.max_speed = {
                        value.as_f64().expect("Wrong JSON settings format") as f32
                    }
                },
                "player_max_accel" => {
                    settings.max_accel = {
                        value.as_f64().expect("Wrong JSON settings format") as f32
                    }
                },
                "air_speed_mult" => {
                    settings.air_speed_mult = {
                        value.as_f64().expect("Wrong JSON settings format") as f32
                    }
                },
                "player_jump_y_speed" => {
                    settings.jump_y_speed = {
                        value.as_f64().expect("Wrong JSON settings format") as f32
                    }
                },
                "player_jump_w_speed" => {
                    settings.jump_w_speed = {
                        value.as_f64().expect("Wrong JSON settings format") as f32
                    }
                },
                "gravity_y_speed" => {
                    settings.gravity_y_speed = {
                        value.as_f64().expect("Wrong JSON settings format") as f32
                    }
                },
                "gravity_w_speed" => {
                    settings.gravity_w_speed = {
                        value.as_f64().expect("Wrong JSON settings format") as f32
                    }
                },
                "player_jetpak_w_speed" => {
                    settings.jetpak_w_speed = {
                        value.as_f64().expect("Wrong JSON settings format") as f32
                    }
                },
                "friction_on_ground" => {
                    settings.friction_on_ground = {
                        value.as_f64().expect("Wrong JSON settings format") as f32
                    }
                },
                "friction_on_air" => {
                    settings.friction_on_air = {
                        value.as_f64().expect("Wrong JSON settings format") as f32
                    }
                },
                _ => {
                    panic!("Wrong JSON settings format")
                }
            }
        }

    settings
}