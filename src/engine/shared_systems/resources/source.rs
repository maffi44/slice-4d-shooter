// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use std::{borrow::Cow, io::ErrorKind, path::Path};

include!(concat!(env!("OUT_DIR"), "/embedded_assets.rs"));

// CLI and map JSON supply level names at runtime. These declarations choose which maps ship.
const _: &[&str] = &[
    crate::embedded_asset!("maps/capture_the_flag_map.json"),
    crate::embedded_asset!("maps/map_2d_3d.json"),
    crate::embedded_asset!("maps/obstacle_course_levels/level-1.json"),
    crate::embedded_asset!("maps/obstacle_course_levels/level-2.json"),
    crate::embedded_asset!("maps/obstacle_course_levels/level-3.json"),
    crate::embedded_asset!("maps/obstacle_course_levels/level-4.json"),
    crate::embedded_asset!("maps/obstacle_course_levels/level-5.json"),
    crate::embedded_asset!("maps/obstacle_course_levels/level-6.json"),
];

pub(super) struct Source {
    pub origin: String,
    pub bytes: Cow<'static, [u8]>,
}

pub(super) enum Lookup {
    CatalogFirst,
    DiskFirst,
}

pub(super) async fn read(
    key: &Path,
    paths: impl IntoIterator<Item = impl AsRef<Path>>,
    lookup: Lookup,
) -> Result<Source, String> {
    let embedded = || {
        // Catalog keys are relative and use forward slashes on every platform.
        let mut name = String::new();
        for component in key.components() {
            match component {
                std::path::Component::Normal(part) => {
                    if !name.is_empty() {
                        name.push('/');
                    }
                    name.push_str(part.to_str()?);
                }
                std::path::Component::CurDir => {}
                _ => return None,
            }
        }
        embedded_bytes(&name).map(|bytes| Source {
            origin: format!("embedded/{name}"),
            bytes: Cow::Borrowed(bytes),
        })
    };
    if matches!(lookup, Lookup::CatalogFirst) {
        if let Some(source) = embedded() {
            return Ok(source);
        }
    }
    for path in paths {
        let path = path.as_ref();
        match tokio::fs::read(path).await {
            Ok(bytes) => {
                return Ok(Source {
                    origin: path.display().to_string(),
                    bytes: Cow::Owned(bytes),
                })
            }
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("{}: {error}", path.display())),
        }
    }
    if matches!(lookup, Lookup::DiskFirst) {
        if let Some(source) = embedded() {
            return Ok(source);
        }
    }
    Err(format!("Cannot find resource {}", key.display()))
}
