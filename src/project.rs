//! NESImg project format

use std::path::PathBuf;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// The actual project structure, as serialized to JSON for the project file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Project {
    pub sources: IndexMap<Ulid, PathBuf>,
}
