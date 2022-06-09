#![warn(clippy::all, rust_2018_idioms)]
#![allow(clippy::reversed_empty_ranges)]

mod cli;
mod constants;
mod gui;
mod project;

use std::marker::PhantomData;

pub use cli::run;
pub use gui::NesimgGui;

/// A wrapper type around a [`Ulid`] that has a generic field for the kind of type the Ulid is meant
/// to refer to.
#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
#[serde(from = "ulid::Ulid")]
#[serde(into = "ulid::Ulid")]
pub struct Uid<T> {
    id: ulid::Ulid,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T> Clone for Uid<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            _phantom: self._phantom.clone(),
        }
    }
}

impl<T> Into<ulid::Ulid> for Uid<T> {
    fn into(self) -> ulid::Ulid {
        self.id
    }
}

impl<T> From<ulid::Ulid> for Uid<T> {
    fn from(id: ulid::Ulid) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }
}

impl<T> Copy for Uid<T> {}

impl<T> std::cmp::PartialEq for Uid<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> std::cmp::Eq for Uid<T> {}

impl<T> std::hash::Hash for Uid<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> Uid<T> {
    pub fn new() -> Self {
        Self {
            id: ulid::Ulid::new(),
            _phantom: PhantomData,
        }
    }
}

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    eframe::start_web(canvas_id, Box::new(|cc| Box::new(NesimgGui::new(cc))))
}
