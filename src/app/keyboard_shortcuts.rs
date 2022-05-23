use std::collections::HashMap;

use egui::{Key, Modifiers};
use once_cell::sync::Lazy;

use super::Action;

#[derive(Debug, Clone)]
pub struct KeyboardShortcut {
    pub modifiers: Modifiers,
    pub key: Key,
}

impl PartialEq for KeyboardShortcut {
    fn eq(&self, other: &Self) -> bool {
        self.modifiers.matches(other.modifiers) && self.key == other.key
    }
}

impl Eq for KeyboardShortcut {}

impl From<(Modifiers, Key)> for KeyboardShortcut {
    fn from((modifiers, key): (Modifiers, Key)) -> Self {
        Self { modifiers, key }
    }
}

impl std::fmt::Display for KeyboardShortcut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.modifiers.mac_cmd {
            write!(f, "⌘ + ")?;
        }

        if self.modifiers.command {
            #[cfg(not(target_os = "macos"))]
            write!(f, "ctrl + ")?;
            #[cfg(target_os = "macos")]
            write!(f, "⌘ + ")?;
        }

        if self.modifiers.alt {
            write!(f, "alt + ")?;
        }

        if self.modifiers.shift {
            write!(f, "shift + ")?;
        }

        write!(f, "{:?}", self.key)?;

        Ok(())
    }
}

pub(crate) static KEYBOARD_SHORTCUTS: Lazy<HashMap<Action, KeyboardShortcut>> = Lazy::new(|| {
    let mut shortcuts = HashMap::default();

    shortcuts.insert(Action::Quit, (Modifiers::COMMAND.into(), Key::Q).into());
    shortcuts.insert(
        Action::LoadImage,
        (Modifiers::COMMAND.into(), Key::O).into(),
    );
    shortcuts.insert(Action::Save, (Modifiers::COMMAND.into(), Key::S).into());
    shortcuts.insert(Action::Export, (Modifiers::COMMAND.into(), Key::E).into());

    shortcuts
});
