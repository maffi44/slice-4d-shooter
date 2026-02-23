// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2025  Timofei Molokov


use std::{env,fs,io};
use std::path::PathBuf;
use winresource::WindowsResource;

fn main() -> io::Result<()> {
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        WindowsResource::new()
            .set_icon("./media/icon.ico")
            .compile()?;

        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let dll_src = manifest_dir.join("dll").join("WinSparkle.dll");
        let target_dir = manifest_dir
            .join("target")
            .join(env::var("TARGET").unwrap())
            .join(env::var("PROFILE").unwrap());

        fs::create_dir_all(&target_dir)?;
        let dll_dst = target_dir.join("WinSparkle.dll");
        fs::copy(&dll_src, &dll_dst)
            .expect("Couldn't copy WinSparkle.dll");

        println!("cargo:rerun-if-changed={}", dll_src.display());
    }
    Ok(())
}