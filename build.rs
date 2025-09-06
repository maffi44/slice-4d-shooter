// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2025  Timofei Molokov

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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