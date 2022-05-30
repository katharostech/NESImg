//! NESImg project format

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The actual project structure, as serialized to JSON for the project file
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Project {
    pub sources: Vec<PathBuf>,
}
