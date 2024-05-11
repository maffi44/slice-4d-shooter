use std::{fs::File, io::Read};
#[cfg(target_arch = "wasm32")]
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

    // pub friction_on_ground: f32,
    pub friction_on_air: f32,
}

impl PlayerSettings {
    pub async fn load_player_settings() -> Self {

        #[cfg(target_arch = "wasm32")]
        {
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
                            
                            return parse_json_into_settings(json_settings);
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
                        "ERROR: the player_settings cannot be loaded, err: {}",
                        val.as_string().unwrap_or("".to_string())
                    );
                }  
            };
        }
        

        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut file = File::open("/home/maffi/Dream/web-engine4d/src/assets/maps/settings.json")
                .expect("Can't find seetings.fson file");
    
            let mut file_content = String::new();
            match file.read_to_string(&mut file_content) {
                Ok(_) => {
                    let json_settings = serde_json::from_str(&file_content)
                        .expect("Can't parse settings.json file");

                    return parse_json_into_settings(json_settings);
                },
                Err(e) => {
                    panic!(
                        "ERROR: the player_settings cannot be loaded, err: {}",
                        e.to_string()
                    );
                }
            }
        }
    }
}

fn parse_json_into_settings(json_settigs: Value) -> PlayerSettings {


    let object = json_settigs
        .as_object()
        .expect("Wrong JSON settings format");

    let collider_radius = {
        object
            .get("player_sphere_radius")
            .expect("Have not player_sphere_radius in settings.json")
            .as_f64()
            .expect("player_sphere_radius is not float value in settings.json")
            as f32
    };

    let max_speed = {
        object
            .get("player_max_speed")
            .expect("Have not player_max_speed in settings.json")
            .as_f64()
            .expect("player_max_speed is not float value in settings.json")
            as f32
    };
    let max_accel = {
        object
            .get("player_max_accel")
            .expect("Have not player_max_accel in settings.json")
            .as_f64()
            .expect("player_max_accel is not float value in settings.json")
            as f32
    };
    let air_speed_mult = {
        object
            .get("air_speed_mult")
            .expect("Have not air_speed_mult in settings.json")
            .as_f64()
            .expect("air_speed_mult is not float value in settings.json")
            as f32
    };
    let jump_y_speed = {
        object
            .get("player_jump_y_speed")
            .expect("Have not player_jump_y_speed in settings.json")
            .as_f64()
            .expect("player_jump_y_speed is not float value in settings.json")
            as f32
    };
    let jump_w_speed = {
        object
            .get("player_jump_w_speed")
            .expect("Have not player_jump_w_speed in settings.json")
            .as_f64()
            .expect("player_jump_w_speed is not float value in settings.json")
            as f32
    };
    let jetpak_w_speed = {
        object
            .get("player_jetpak_w_speed")
            .expect("Have not player_jetpak_w_speed in settings.json")
            .as_f64()
            .expect("player_jetpak_w_speed is not float value in settings.json")
            as f32
    };
    let gravity_y_speed = {
        object
            .get("gravity_y_speed")
            .expect("Have not gravity_y_speed in settings.json")
            .as_f64()
            .expect("gravity_y_speed is not float value in settings.json")
            as f32
    };
    let gravity_w_speed = {
        object
            .get("gravity_w_speed")
            .expect("Have not gravity_w_speed in settings.json")
            .as_f64()
            .expect("gravity_w_speed is not float value in settings.json")
            as f32
    };
    let friction_on_air = {
        object
            .get("friction_on_air")
            .expect("Have not friction_on_air in settings.json")
            .as_f64()
            .expect("friction_on_air is not float value in settings.json")
            as f32
    };

    PlayerSettings {
        collider_radius,
        max_speed,
        max_accel,
        air_speed_mult,
        jump_y_speed,
        jump_w_speed,
        jetpak_w_speed,
        gravity_y_speed,
        gravity_w_speed,
        friction_on_air,
    }
}