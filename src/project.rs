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
    /// The metatiles
    pub metatiles: IndexMap<Ulid, Metatile>,
    /// The metatilesets
    pub metatilesets: IndexMap<Ulid, Metatileset>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct Metatileset {
    /// A human-readable name for reference purposes
    pub name: String,

    /// The color pallet used to render the metatileset tiles
    pub pallet: Pallet,

    /// The metatiles that make up the metatileset
    pub tiles: Vec<MetatilesetTile>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct MetatilesetTile {
    /// The id of the metatile.
    id: Ulid,
    /// The index in the range `0..4` of the sub-pallet to use for rendering the metatile.
    pallet: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct Pallet {
    /// The 13 colors that make up the pallet.
    ///
    /// When used for backgrounds, the first color is the background color, when used for sprites,
    /// the first color is effectively transparent during rendering, regardless of what it is
    /// actually set to.
    ///
    /// The colors are indexes into the NES color pallet: [`crate::constants::NES_PALLET`]
    pub colors: [u8; 13],
}

impl Pallet {
    /// Returns a slice of 4 sub-pallets, each with four colors. This mirrors the first of the 13
    /// colors to the first color of each of the sub-pallets, just like the NES will.
    pub fn get_sub_pallets(&self) -> [[u8; 4]; 4] {
        let c = self.colors;
        [
            [c[0], c[1], c[2], c[3]],
            [c[0], c[4], c[5], c[6]],
            [c[0], c[7], c[8], c[9]],
            [c[0], c[10], c[11], c[12]],
        ]
    }
}
