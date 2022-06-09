//! NESImg project format

use std::path::PathBuf;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::Uid;

/// The actual project structure, as serialized to JSON for the project file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct Project {
    /// The source images
    pub sources: IndexMap<Uid<PathBuf>, PathBuf>,
    /// The metatiles
    pub metatiles: IndexMap<Uid<Metatile>, Metatile>,
    /// The metatilesets
    pub metatilesets: IndexMap<Uid<Metatileset>, Metatileset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct Tile {
    pub source_id: Uid<PathBuf>,
    /// The x tile index in the source image
    pub x: u16,
    /// The y tile index in the sorce image
    pub y: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct Metatile {
    /// The tiles that make up the metatile
    pub tiles: [Option<Tile>; 4],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct Metatileset {
    /// A human-readable name for reference purposes
    pub name: String,

    /// The color pallet used to render the metatileset tiles
    pub pallet: Pallet,

    /// The metatiles that make up the metatileset
    pub tiles: IndexMap<Uid<MetatilesetTile>, MetatilesetTile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct MetatilesetTile {
    /// The id of the metatile.
    pub metatile_id: Uid<Metatile>,
    /// The index in the range `0..4` of the sub-pallet to use for rendering the metatile.
    pub sub_pallet_idx: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Pallet {
    /// The 13 colors that make up the pallet.
    ///
    /// When used for backgrounds, the first color is the background color, when used for sprites,
    /// the first color is effectively transparent during rendering, regardless of what it is
    /// actually set to.
    ///
    /// The colors are indexes into the NES color pallet: [`crate::constants::NES_PALLET`]
    pub colors: [u32; 13],
}

impl Default for Pallet {
    fn default() -> Self {
        Self {
            colors: [
                0x0f, 0x2d, 0x10, 0x30, 0x2d, 0x10, 0x30, 0x2d, 0x10, 0x30, 0x2d, 0x10, 0x30,
            ],
        }
    }
}

impl Pallet {
    /// Returns a slice of 4 sub-pallets, each with four colors. This mirrors the first of the 13
    /// colors to the first color of each of the sub-pallets, just like the NES will.
    pub fn get_sub_pallets(&self) -> [[u32; 4]; 4] {
        let c = self.colors;
        [
            [c[0], c[1], c[2], c[3]],
            [c[0], c[4], c[5], c[6]],
            [c[0], c[7], c[8], c[9]],
            [c[0], c[10], c[11], c[12]],
        ]
    }
}
