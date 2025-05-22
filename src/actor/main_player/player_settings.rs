use std::{fs::File, io::Read, sync::{Arc, Mutex}};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
use serde_json::Value;

#[derive(Clone)]
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

    pub friction_on_air: f32,

    pub rotation_along_w_standard_method: bool,
    pub shadows_enable: bool,
    
    pub mouse_sensivity: Arc<Mutex<f32>>,
    pub w_jump_time_reloading: f32,
    pub min_respawn_timer: f32,
    pub max_respawn_timer: f32,
    pub scanner_show_enemies_time: f32,
    pub scanner_reloading_time: f32,
    
    pub energy_gun_hole_size_mult: f32, 
    pub energy_gun_add_force_mult: f32, 
    pub energy_gun_damage_mult: f32, 
    pub energy_gun_restoring_speed: f32,
    
    pub machinegun_damage: f32,
    pub machinegun_add_force: f32, 
    pub machinegun_heat_add_on_shot: f32, 
    pub machinegun_cooling_speed: f32,

    pub matchmaking_server_url: String,
    pub bash_and_turn_servers: Vec<String>,
    pub turn_server_username: String,
    pub turn_server_credential: String,

    pub screen_resolution_scale: f32,
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
            let mut file = File::open("./src/assets/maps/settings.json");

            if file.is_err()
            {
                file = File::open("../../../src/assets/maps/settings.json");
            }

            match file {
                Ok(mut file) =>
                {
                    let mut file_content = String::new();
                    match file.read_to_string(&mut file_content) {
                        Ok(_) => {
                            let json_settings = serde_json::from_str(&file_content)
                                .expect("Can't parse settings.json file");

                            let json_settings2: Value = serde_json::from_str(include_str!("../../../src/assets/maps/settings2.json"))
                                .expect("Can't parse settings2.json file");

                            return parse_json_into_settings(json_settings, json_settings2);
                        },
                        Err(_) => {
                            let json_settings = serde_json::from_str(include_str!("../../../src/assets/maps/settings.json"))
                                .expect("Can't parse settings.json file");
        
                            let json_settings2: Value = serde_json::from_str(include_str!("../../../src/assets/maps/settings2.json"))
                                .expect("Can't parse settings2.json file");
        
                            return parse_json_into_settings(json_settings, json_settings2);
                        }
                    }
                }
                Err(_) =>
                {
                    let json_settings = serde_json::from_str(include_str!("../../../src/assets/maps/settings.json"))
                        .expect("Can't parse settings.json file");

                    let json_settings2: Value = serde_json::from_str(include_str!("../../../src/assets/maps/settings2.json"))
                        .expect("Can't parse settings2.json file");

                    return parse_json_into_settings(json_settings, json_settings2);
                }
            }
        }
    }
}

fn parse_json_into_settings(json_settigs: Value, json_settigs2: Value) -> PlayerSettings {


    let object = json_settigs
        .as_object()
        .expect("Wrong JSON settings format");

    let object2 = json_settigs2
        .as_object()
        .expect("Wrong JSON settings format");

    let collider_radius = {
        object
            .get("player_sphere_radius")
            .unwrap_or_else(||{
                object2.get("player_sphere_radius").unwrap()
            })
            .as_f64()
            .expect("player_sphere_radius is not float value in settings.json")
            as f32
    };

    let max_speed = {
        object
            .get("player_max_speed")
            .unwrap_or_else(||{
                object2.get("player_max_speed").unwrap()
            })
            .as_f64()
            .expect("player_max_speed is not float value in settings.json")
            as f32
    };
    let max_accel = {
        object
            .get("player_max_accel")
            .unwrap_or_else(||{
                object2.get("player_max_accel").unwrap()
            })
            .as_f64()
            .expect("player_max_accel is not float value in settings.json")
            as f32
    };
    let air_speed_mult = {
        object
            .get("air_speed_mult")
            .unwrap_or_else(||{
                object2.get("air_speed_mult").unwrap()
            })
            .as_f64()
            .expect("air_speed_mult is not float value in settings.json")
            as f32
    };
    let jump_y_speed = {
        object
            .get("player_jump_y_speed")
            .unwrap_or_else(||{
                object2.get("player_jump_y_speed").unwrap()
            })
            .as_f64()
            .expect("player_jump_y_speed is not float value in settings.json")
            as f32
    };
    let jump_w_speed = {
        object
            .get("player_jump_w_speed")
            .unwrap_or_else(||{
                object2.get("player_jump_w_speed").unwrap()
            })
            .as_f64()
            .expect("player_jump_w_speed is not float value in settings.json")
            as f32
    };
    let jetpak_w_speed = {
        object
            .get("player_jetpack_w_speed")
            .unwrap_or_else(||{
                object2.get("player_jetpack_w_speed").unwrap()
            })
            .as_f64()
            .expect("player_jetpak_w_speed is not float value in settings.json")
            as f32
    };
    let gravity_y_speed = {
        object
            .get("gravity_y_speed")
            .unwrap_or_else(||{
                object2.get("gravity_y_speed").unwrap()
            })
            .as_f64()
            .expect("gravity_y_speed is not float value in settings.json")
            as f32
    };
    let gravity_w_speed = {
        object
            .get("gravity_w_speed")
            .unwrap_or_else(||{
                object2.get("gravity_w_speed").unwrap()
            })
            .as_f64()
            .expect("gravity_w_speed is not float value in settings.json")
            as f32
    };
    let friction_on_air = {
        object
            .get("friction_on_air")
            .unwrap_or_else(||{
                object2.get("friction_on_air").unwrap()
            })
            .as_f64()
            .expect("friction_on_air is not float value in settings.json")
            as f32
    };
    let rotation_along_w_standard_method = {
        object
            .get("rotation_along_w_standard_method")
            .unwrap_or_else(||{
                object2.get("rotation_along_w_standard_method").unwrap()
            })
            .as_bool()
            .expect("rotation_along_w_standard_method is not bool value in settings.json")
    };
    let shadows_enable = {
        object
            .get("shadows_enable")
            .unwrap_or_else(||{
                object2.get("shadows_enable").unwrap()
            })
            .as_bool()
            .expect("shadows_enable is not float value in settings.json")
    };
    let mouse_sensivity = {
        object
        .get("mouse_sensivity")
        .unwrap_or_else(||{
            object2.get("mouse_sensivity").unwrap()
        })
        .as_f64()
        .expect("mouse_sensivity is not float value in settings.json")
        as f32
    };
    let w_jump_time_reloading = {
        object
        .get("w_jump_time_reloading")
        .unwrap_or_else(||{
            object2.get("w_jump_time_reloading").unwrap()
        })
        .as_f64()
        .expect("w_jump_time_reloading is not float value in settings.json")
        as f32
    };
    let min_respawn_timer = {
        object
        .get("min_respawn_timer")
        .unwrap_or_else(||{
            object2.get("min_respawn_timer").unwrap()
        })
        .as_f64()
        .expect("min_respawn_timer is not float value in settings.json")
        as f32
    };
    let max_respawn_timer = {
        object
        .get("max_respawn_timer")
        .unwrap_or_else(||{
            object2.get("max_respawn_timer").unwrap()
        })
        .as_f64()
        .expect("max_respawn_timer is not float value in settings.json")
        as f32
    };
    let scanner_reloading_time = {
        object
        .get("scanner_reloading_time")
        .unwrap_or_else(||{
            object2.get("scanner_reloading_time").unwrap()
        })
        .as_f64()
        .expect("scanner_reloading_time is not float value in settings.json")
        as f32
    };
    let scanner_show_enemies_time = {
        object
        .get("scanner_show_enemies_time")
        .unwrap_or_else(||{
            object2.get("scanner_show_enemies_time").unwrap()
        })
        .as_f64()
        .expect("scanner_show_enemies_time is not float value in settings.json")
        as f32
    };
    let energy_gun_hole_size_mult = {
        object
        .get("energy_gun_hole_size_mult")
        .unwrap_or_else(||{
            object2.get("energy_gun_hole_size_mult").unwrap()
        })
        .as_f64()
        .expect("energy_gun_hole_size_mult is not float value in settings.json")
        as f32
    }; 
    let energy_gun_add_force_mult = {
        object
        .get("energy_gun_add_force_mult")
        .unwrap_or_else(||{
            object2.get("energy_gun_add_force_mult").unwrap()
        })
        .as_f64()
        .expect("energy_gun_add_force_mult is not float value in settings.json")
        as f32
    }; 
    let energy_gun_damage_mult = {
        object
        .get("energy_gun_damage_mult")
        .unwrap_or_else(||{
            object2.get("energy_gun_damage_mult").unwrap()
        })
        .as_f64()
        .expect("energy_gun_damage_mult is not float value in settings.json")
        as f32
    }; 
    let energy_gun_restoring_speed = {
        object
        .get("energy_gun_restoring_speed")
        .unwrap_or_else(||{
            object2.get("energy_gun_restoring_speed").unwrap()
        })
        .as_f64()
        .expect("energy_gun_restoring_speed is not float value in settings.json")
        as f32
    };
    let machinegun_damage = {
        object
        .get("machinegun_damage")
        .unwrap_or_else(||{
            object2.get("machinegun_damage").unwrap()
        })
        .as_f64()
        .expect("machinegun_damage is not float value in settings.json")
        as f32
    };
    let machinegun_add_force = {
        object
        .get("machinegun_add_force")
        .unwrap_or_else(||{
            object2.get("machinegun_add_force").unwrap()
        })
        .as_f64()
        .expect("machinegun_add_force is not float value in settings.json")
        as f32
    }; 
    let machinegun_heat_add_on_shot = {
        object
        .get("machinegun_heat_add_on_shot")
        .unwrap_or_else(||{
            object2.get("machinegun_heat_add_on_shot").unwrap()
        })
        .as_f64()
        .expect("machinegun_heat_add_on_shot is not float value in settings.json")
        as f32
    }; 
    let machinegun_cooling_speed = {
        object
        .get("machinegun_cooling_speed")
        .unwrap_or_else(||{
            object2.get("machinegun_cooling_speed").unwrap()
        })
        .as_f64()
        .expect("machinegun_cooling_speed is not float value in settings.json")
        as f32
    };
    let matchmaking_server_url = {
        object
        .get("matchmaking_server_url")
        .unwrap_or_else(||{
            object2.get("matchmaking_server_url").unwrap()
        })
        .as_str()
        .expect("matchmaking_server_url is not string value in settings.json")
        .to_string()
    };

    let bash_and_turn_servers = {
        object
        .get("bash_and_turn_servers")
        .unwrap_or_else(||{
            object2.get("bash_and_turn_servers").unwrap()
        })
        .as_array()
        .expect("bash_and_turn_servers is not array value in settings.json")
        .into_iter()
        .map(|e| e.as_str().expect("bash_and_turn_servers array element is not a string").to_string())
        .collect()
    };

    let turn_server_username = {
        object
        .get("turn_server_username")
        .unwrap_or_else(||{
            object2.get("turn_server_username").unwrap()
        })
        .as_str()
        .expect("turn_server_username is not string value in settings.json")
        .to_string()
    };

    let turn_server_credential = {
        object
        .get("turn_server_credential")
        .unwrap_or_else(||{
            object2.get("turn_server_credential").unwrap()
        })
        .as_str()
        .expect("turn_server_credential is not string value in settings.json")
        .to_string()
    };

    let screen_resolution_scale = {
        object
        .get("screen_resolution_scale")
        .unwrap_or_else(||{
            object2.get("screen_resolution_scale").unwrap()
        })
        .as_f64()
        .expect("screen_resolution_scale is not number value in settings.json")
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
        rotation_along_w_standard_method,
        shadows_enable,
        mouse_sensivity: Arc::new(Mutex::new(mouse_sensivity)),
        w_jump_time_reloading,
        min_respawn_timer,
        max_respawn_timer,
        scanner_show_enemies_time,
        scanner_reloading_time,
        energy_gun_hole_size_mult, 
        energy_gun_add_force_mult, 
        energy_gun_damage_mult, 
        energy_gun_restoring_speed,
        machinegun_damage,
        machinegun_add_force, 
        machinegun_heat_add_on_shot, 
        machinegun_cooling_speed,
        matchmaking_server_url,
        bash_and_turn_servers,
        turn_server_username,
        turn_server_credential,
        screen_resolution_scale,
    }
}