//! NESImg project format

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Project {
    sources: Vec<String>,
}
