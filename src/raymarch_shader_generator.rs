mod engine;
mod actor;
mod main_loop;
mod transform;


use std::{fs::{File, OpenOptions}, io::Write};

use crate::{actor::main_player::player_settings::PlayerSettings, engine::{engine_handle::EngineHandle, render::{raymarch_shader_generator, render_data::static_render_data::StaticRenderData}, world::World}};

fn main() {
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("./src/engine/render/shaders/generated_raymarch_shader.wgsl");

    let mut engine_handle = EngineHandle::new();

    let players_settings = pollster::block_on(
        PlayerSettings::load_player_settings()
    );

    let world = pollster::block_on(
        World::new(
            &mut engine_handle,
            players_settings,
            "map".to_string()
        )
    );

    let static_data = StaticRenderData::new(&world);

    match file {
        Ok(mut file) => {
            let res = file.write_all(
                raymarch_shader_generator::generate_raymarch_shader_with_static_bsp_tree(
                    include_str!("engine/render/shaders/raymarch_shader.wgsl"),
                    &static_data
                ).as_bytes()
            );

            match res {
                Ok(_) => println!("succes! Raymarch shader generated!"),
                Err(e) => panic!("Raymarch shader generator failture: {}", e),
            }
        },

        Err(e) => panic!("Raymarch shader generator failture: {}", e),
    }
}