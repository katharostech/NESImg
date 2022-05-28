//! NESImg project format

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    sources: Vec<String>,
}
