//! NESImg project format

use std::path::PathBuf;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// The actual project structure, as serialized to JSON for the project file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct Project {
    /// The source images
    pub sources: IndexMap<Ulid, PathBuf>,
    /// The metatiles in the project
    pub metatiles: IndexMap<Ulid, Metatile>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct Tile {
    pub source_id: Ulid,
    /// The x tile index in the source image
    pub x: u16,
    /// The y tile index in the sorce image
    pub y: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct Metatile {
    /// The tiles that make up the metatile
    pub tiles: [Option<Tile>; 4],
}
