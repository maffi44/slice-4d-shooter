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

use wgpu::Backend;

pub fn read_args() -> Option<Backend>
{
    let args: Vec<String> = std::env::args().collect();
    let mut backend = None;
    let mut show_help = false;

    let mut i = 1;
    while i < args.len()
    {
        match args[i].as_str()
        {
            "-b" | "--backend" =>
            {
                if i + 1 < args.len()
                {
                    backend = match args[i + 1].to_lowercase().as_str()
                    {
                        "gl" => Some(Backend::Gl),
                        
                        "dx12" => Some(Backend::Dx12),
                        
                        "vulkan" => Some(Backend::Vulkan),
                        
                        "metal" => Some(Backend::Metal),
                        
                        _ =>
                        {
                            eprintln!("Unknown graphics backend: {}", args[i + 1]);
                            
                            show_help = true;
                            
                            None
                        }
                    };
                    i += 1;
                }
                else
                {
                    eprintln!("Missing argument for backend option");
                }
            }

            "--help" | "-help" | "help" | "-h" | "--usage" | "-usage" | "usage" =>
            {
                show_help = true;
            }

            "-v" | "--v" | "-version" | "--version" =>
            {
                println!("Slice: 4D Shooter version: {}", env!("CARGO_PKG_VERSION"));
                
                std::process::exit(0);

            }

            _ =>
            {
                eprintln!("Unknown argument: {}", args[i]);

                show_help = true;
            }
        }

        i += 1;
    }

    if show_help
    {
        println!("Usage: {} [OPTIONS]", "{}");
        println!();
        println!("Options:");
        println!("  -b, --backend BACKEND        Set graphics backend (gl, dx12, vulkan, metal)");
        println!("  --help, -help, -h            Show this help message");
        println!("  -v --v -version, --version,  Show current game version");
        println!();
        println!("If no backend is specified, the graphics backend will be selected automatically");

        std::process::exit(0);
    }

    backend
}