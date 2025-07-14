mod engine;
mod actor;
mod main_loop;
mod transform;


use std::{env, fs::OpenOptions, io::{Read, Write}};

use crate::{actor::main_player::player_settings::PlayerSettings, engine::{engine_handle::EngineHandle, render::{raymarch_shader_generator, render_data::static_render_data::StaticRenderData}, world::World}};

// This is used for generate raymarch shader with a Static BSP Tree

fn main() {
    let args: Vec<String> = env::args().collect();

    let (mut original_shader_file, mut file_for_write, map_name) =
        if let Some(shader_name) = args.get(1)
    {
        let original_shader_file = OpenOptions::new()
            .read(true)
            .open(format!("./src/engine/render/shaders/{}.wgsl", shader_name))
            .expect("Can't open original ray march shader file on given path");
        
        let file_for_write = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(format!("./src/engine/render/shaders/{}_with_bsp_tree.wgsl", shader_name))
            .expect("can't open the file to write a new generated shader to it");

        let map_name = if shader_name == "raymarch_shader_for_2d_3d_example"
        {
            "map_2d_3d"
        }
        else
        {
            "map"
        };

        (original_shader_file, file_for_write, map_name)
    }
    else
    {
        let original_shader_file = OpenOptions::new()
            .read(true)
            .open("./src/engine/render/shaders/raymarch_shader.wgsl")
            .expect("Can't open original ray march shader file on ./src/engine/render/shaders/raymarch_shader.wgsl");

        let file_for_write = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open("./src/engine/render/shaders/raymarch_shader_with_bsp_tree.wgsl")
            .expect("can't open the file to write a new generated shader to it");

        (original_shader_file, file_for_write, "map")
    };

    let mut original_shader = String::new();
    
    original_shader_file
        .read_to_string(&mut original_shader)
        .unwrap();


    let mut engine_handle = EngineHandle::new();

    let players_settings = pollster::block_on(
        PlayerSettings::load_player_settings()
    );

    let world = pollster::block_on(
        World::new(
            &mut engine_handle,
            players_settings,
            map_name.to_string()
        )
    );

    let static_data = StaticRenderData::new(&world);
    
    let res = file_for_write.write_all(
        raymarch_shader_generator::generate_raymarch_shader_with_static_bsp_tree(
            &original_shader,
            &static_data
        ).as_bytes()
    );

    match res {
        Ok(_) => println!("succes! Raymarch shader generated!"),
        Err(e) => panic!("Raymarch shader generator failure: {}", e),
    }
}