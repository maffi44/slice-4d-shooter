// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use wgpu::Backend;

pub struct Args
{
    pub graphic_backend: Option<Backend>,
    pub specific_level: Option<String>,
    /// `--headless`: no window, GPU, rendering or sound.
    pub headless: bool,
}

pub fn read_args() -> Args
{
    let args: Vec<String> = std::env::args().collect();
    let mut backend = None;
    let mut show_help = false;
    let mut level = None;
    let mut headless = false;

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
                        "gl" =>
                        {
                            eprintln!("OpenGL is no longer supported. Use a supported graphics backend.");
                            std::process::exit(2);
                        }
                        
                        "dx12" => Some(Backend::Dx12),

                        "webgpu" =>
                        {
                            eprintln!("WebGPU requires a browser build; this client is native. Use vulkan, dx12 or metal.");
                            std::process::exit(2);
                        }
                        
                        "vulkan" => Some(Backend::Vulkan),
                        
                        "metal" => Some(Backend::Metal),
                        
                        _ =>
                        {
                            eprintln!("Unknown graphics backend: {}", args[i + 1]);
                            
                            std::process::exit(2);
                        }
                    };
                    i += 1;
                }
                else
                {
                    eprintln!("Missing argument for backend option");

                    std::process::exit(2);
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

            "--headless" | "-headless" =>
            {
                headless = true;
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
        println!("  -b, --backend BACKEND        Set graphics backend (vulkan, dx12, metal)");
        println!("  --help, -help, -h            Show this help message");
        println!("  -v --v -version, --version,  Show current game version");
        println!("  -l --l -level, --level,  Load specific level");
        println!("  --headless, -headless    Run without window, rendering and sound");
        println!();
        println!("If no backend is specified, the graphics backend will be selected automatically");

        std::process::exit(0);
    }

    Args
    {
        graphic_backend: backend,
        specific_level: level,
        headless,
    }
}
