// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use wgpu::Backend;

#[derive(Clone, Copy)]
pub enum GameMode
{
    NORMAL,
    OBSERVER,
}

impl Default for GameMode
{
    fn default() -> Self {
        GameMode::OBSERVER
    }
}

pub struct Args
{
    pub graphic_backend: Option<Backend>,
    pub specific_level: Option<String>,
    pub game_mode: GameMode,
}

pub fn read_args() -> Args
{
    let args: Vec<String> = std::env::args().collect();
    let mut backend = None;
    let mut show_help = false;
    let mut level = None;
    let mut mode = GameMode::NORMAL;

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

                    std::process::exit(0);
                }
            }

            "--help" | "-help" | "help" | "-h" | "--usage" | "-usage" | "usage" =>
            {
                show_help = true;
            }

            "-l" | "-level" | "--l" | "--level" =>
            {
                if i + 1 < args.len()
                {
                    level = Some(args[i + 1].to_string());
                    i += 1;
                }
                else
                {
                    eprintln!("Missing argument for level option");

                    std::process::exit(0);
                }
            }

            "-m" | "-mode" | "--m" | "--mode" =>
            {
                if i + 1 < args.len()
                {
                    mode = match args[i + 1].to_lowercase().as_str()
                    {
                        "observer" => GameMode::OBSERVER,

                        "normal" => GameMode::NORMAL,
                        
                        _ =>
                        {
                            eprintln!("Unknown game mode: {}", args[i + 1]);
                            
                            show_help = true;

                            GameMode::OBSERVER
                        }
                    };
                    i += 1;
                }
                else
                {
                    eprintln!("Missing argument for level option");

                    std::process::exit(0);
                }
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
        println!("  -l --l -level, --level,  Load specific level");
        println!("  -m --m -mode, --mode,  Load specific game mode (normal, observer)");
        println!();
        println!("If no backend is specified, the graphics backend will be selected automatically");

        std::process::exit(0);
    }

    Args
    {
        graphic_backend: backend,
        specific_level: level,
        game_mode: mode     
    }
}