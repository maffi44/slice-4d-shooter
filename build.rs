// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use proc_macro2::{TokenStream, TokenTree};
use std::{
    collections::BTreeSet,
    env, fs, io,
    io::Write,
    path::{Path, PathBuf},
};
use winresource::WindowsResource;

fn collect_markers(tokens: TokenStream, keys: &mut BTreeSet<String>) -> Result<(), String> {
    let mut tokens = tokens.into_iter().peekable();
    while let Some(token) = tokens.next() {
        match token {
            TokenTree::Ident(name)
                if name == "embedded_asset"
                    && tokens
                        .next_if(|t| matches!(t, TokenTree::Punct(p) if p.as_char() == '!'))
                        .is_some() =>
            {
                let Some(TokenTree::Group(args)) = tokens.next() else {
                    return Err("expected embedded_asset!(\"relative/path\")".into());
                };
                let literal = syn::parse2::<syn::LitStr>(args.stream())
                    .map_err(|_| "embedded_asset! requires one string literal")?;
                let key = literal.value();
                if key.contains(['\\', ':']) || key.split('/').any(|p| matches!(p, "" | "." | ".."))
                {
                    return Err(format!(
                        "invalid embedded asset path {key:?}, expected a relative /-separated path"
                    ));
                }
                keys.insert(key);
            }
            TokenTree::Group(group) => collect_markers(group.stream(), keys)?,
            _ => {}
        }
    }
    Ok(())
}

fn scan_sources(dir: &Path, keys: &mut BTreeSet<String>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            scan_sources(&path, keys)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let source = fs::read_to_string(&path)?;
            let tokens = source
                .parse::<TokenStream>()
                .map_err(|e| io::Error::other(format!("{}: {e}", path.display())))?;
            collect_markers(tokens, keys)
                .map_err(|e| io::Error::other(format!("{}: {e}", path.display())))?;
        }
    }
    Ok(())
}

fn generate_asset_catalog() -> io::Result<()> {
    println!("cargo:rerun-if-changed=src");
    let mut keys = BTreeSet::new();
    scan_sources(Path::new("src"), &mut keys)?;
    let mut output = Vec::new();
    writeln!(
        output,
        "fn embedded_bytes(key: &str) -> Option<&'static [u8]> {{ match key {{"
    )?;
    for key in keys {
        writeln!(
            output,
            "{:?} => Some(include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), {:?}))),",
            format!("embedded/{key}"),
            format!("/src/assets/{key}")
        )?;
    }
    writeln!(output, "_ => None, }} }}")?;
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("embedded_assets.rs"),
        output,
    )
}

fn main() -> io::Result<()> {
    generate_asset_catalog()?;
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        println!("cargo:rerun-if-changed=media/icon.ico");
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
        fs::copy(&dll_src, &dll_dst).expect("Couldn't copy WinSparkle.dll");

        println!("cargo:rerun-if-changed={}", dll_src.display());
    }
    Ok(())
}
