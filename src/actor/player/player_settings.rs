use std::{fs::File, io::Read};
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
    
    pub mouse_sensivity: f32,
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
            let mut file = File::open("./src/assets/maps/settings.json")
                .unwrap_or_else(|_| {
                    File::open("/home/maffi/Dream/web-engine4d/src/assets/maps/settings.json").expect("Can't find settings.json file")
                });
    
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
    let rotation_along_w_standard_method = {
        object
            .get("rotation_along_w_standard_method")
            .expect("Have not rotation_along_w_standard_method in settings.json")
            .as_bool()
            .expect("rotation_along_w_standard_method is not bool value in settings.json")
    };
    let shadows_enable = {
        object
            .get("shadows_enable")
            .expect("Have not shadows_enable in settings.json")
            .as_bool()
            .expect("shadows_enable is not float value in settings.json")
    };
    let mouse_sensivity = {
        object
        .get("mouse_sensivity")
        .expect("Have not mouse_sensivity in settings.json")
        .as_f64()
        .expect("mouse_sensivity is not float value in settings.json")
        as f32
    };
    let w_jump_time_reloading = {
        object
        .get("w_jump_time_reloading")
        .expect("Have not w_jump_time_reloading in settings.json")
        .as_f64()
        .expect("w_jump_time_reloading is not float value in settings.json")
        as f32
    };
    let min_respawn_timer = {
        object
        .get("min_respawn_timer")
        .expect("Have not min_respawn_timer in settings.json")
        .as_f64()
        .expect("min_respawn_timer is not float value in settings.json")
        as f32
    };
    let max_respawn_timer = {
        object
        .get("max_respawn_timer")
        .expect("Have not max_respawn_timer in settings.json")
        .as_f64()
        .expect("max_respawn_timer is not float value in settings.json")
        as f32
    };
    let scanner_reloading_time = {
        object
        .get("scanner_reloading_time")
        .expect("Have not scanner_reloading_time in settings.json")
        .as_f64()
        .expect("scanner_reloading_time is not float value in settings.json")
        as f32
    };
    let scanner_show_enemies_time = {
        object
        .get("scanner_show_enemies_time")
        .expect("Have not scanner_show_enemies_time in settings.json")
        .as_f64()
        .expect("scanner_show_enemies_time is not float value in settings.json")
        as f32
    };
    let energy_gun_hole_size_mult = {
        object
        .get("energy_gun_hole_size_mult")
        .expect("Have not energy_gun_hole_size_mult in settings.json")
        .as_f64()
        .expect("energy_gun_hole_size_mult is not float value in settings.json")
        as f32
    }; 
    let energy_gun_add_force_mult = {
        object
        .get("energy_gun_add_force_mult")
        .expect("Have not energy_gun_add_force_mult in settings.json")
        .as_f64()
        .expect("energy_gun_add_force_mult is not float value in settings.json")
        as f32
    }; 
    let energy_gun_damage_mult = {
        object
        .get("energy_gun_damage_mult")
        .expect("Have not energy_gun_damage_mult in settings.json")
        .as_f64()
        .expect("energy_gun_damage_mult is not float value in settings.json")
        as f32
    }; 
    let energy_gun_restoring_speed = {
        object
        .get("energy_gun_restoring_speed")
        .expect("Have not energy_gun_restoring_speed in settings.json")
        .as_f64()
        .expect("energy_gun_restoring_speed is not float value in settings.json")
        as f32
    };
    let machinegun_damage = {
        object
        .get("machinegun_damage")
        .expect("Have not machinegun_damage in settings.json")
        .as_f64()
        .expect("machinegun_damage is not float value in settings.json")
        as f32
    };
    let machinegun_add_force = {
        object
        .get("machinegun_add_force")
        .expect("Have not machinegun_add_force in settings.json")
        .as_f64()
        .expect("machinegun_add_force is not float value in settings.json")
        as f32
    }; 
    let machinegun_heat_add_on_shot = {
        object
        .get("machinegun_heat_add_on_shot")
        .expect("Have not machinegun_heat_add_on_shot in settings.json")
        .as_f64()
        .expect("machinegun_heat_add_on_shot is not float value in settings.json")
        as f32
    }; 
    let machinegun_cooling_speed = {
        object
        .get("machinegun_cooling_speed")
        .expect("Have not machinegun_cooling_speed in settings.json")
        .as_f64()
        .expect("machinegun_cooling_speed is not float value in settings.json")
        as f32
    };
    let matchmaking_server_url = {
        object
        .get("matchmaking_server_url")
        .expect("Have not matchmaking_server_url in settings.json")
        .as_str()
        .expect("matchmaking_server_url is not string value in settings.json")
        .to_string()
    };

    let bash_and_turn_servers = {
        object
        .get("bash_and_turn_servers")
        .expect("Have not bash_and_turn_servers in settings.json")
        .as_array()
        .expect("bash_and_turn_servers is not array value in settings.json")
        .into_iter()
        .map(|e| e.as_str().expect("bash_and_turn_servers array element is not a string").to_string())
        .collect()
    };

    let turn_server_username = {
        object
        .get("turn_server_username")
        .expect("Have not turn_server_username in settings.json")
        .as_str()
        .expect("turn_server_username is not string value in settings.json")
        .to_string()
    };

    let turn_server_credential = {
        object
        .get("turn_server_credential")
        .expect("Have not turn_server_credential in settings.json")
        .as_str()
        .expect("turn_server_credential is not string value in settings.json")
        .to_string()
    };

    let screen_resolution_scale = {
        object
        .get("screen_resolution_scale")
        .expect("Have not screen_resolution_scale in settings.json")
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
        mouse_sensivity,
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