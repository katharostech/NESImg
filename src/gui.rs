use eframe::egui;
use egui::{util::undoer::Undoer, Key, Layout, Modifiers, Ui};
use native_dialog::FileDialog;
use notify::Watcher;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    time::{Duration, Instant},
};

use tracing as trc;

mod components;
mod keyboard_shortcuts;
mod tabs;

use components::{send_error_notification, show_notifications};
use keyboard_shortcuts::KeyboardShortcut;
use tabs::NesimgGuiTab;

use crate::project::Project;

/// Run the GUI
pub fn run_gui() {
    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "NESImg",
        native_options,
        Box::new(|cc| Box::new(NesimgGui::new(cc))),
    );
}

/// The root GUI element: renders the menu bar and tabs, and offloads rending the main region to the
/// specific tab gui implementations.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct NesimgGui {
    /// The current GUI tab
    current_tab: String,
    /// The list of tab implementations and their names
    #[serde(skip)]
    tabs: Vec<(String, Box<dyn NesimgGuiTab>)>,

    /// Dark mode enabled state
    dark_mode: bool,

    /// The root GUI state, which will be shared with and allowed to be modified by tabs
    #[serde(skip)]
    state: RootState,
}

impl Default for NesimgGui {
    fn default() -> Self {
        Self {
            dark_mode: true,
            current_tab: "Sources".into(),
            tabs: vec![
                ("Maps".into(), Box::new(tabs::maps::MapsTab::default())),
                (
                    "Namepages".into(),
                    Box::new(tabs::namepages::NamepagesTab::default()),
                ),
                (
                    "Metatiles".into(),
                    Box::new(tabs::metatiles::MetatilesTab::default()),
                ),
                (
                    "Sources".into(),
                    Box::new(tabs::sources::SourcesTab::default()),
                ),
            ],
            state: Default::default(),
        }
    }
}

/// The root GUI state, which will be shared with and allowed to be modified by tabs
pub struct RootState {
    /// The loaded NESImg project, if any
    project: Option<ProjectData>,

    /// Start time of the app, which can be used for calculating elapsed time for [`Undoer`]s
    start: Instant,

    open_image_request_sender: flume::Sender<&'static str>,
    open_image_response_receiver: flume::Receiver<(&'static str, PathBuf)>,
    file_watcher_path_change_sender: std::sync::mpsc::Sender<PathBuf>,
    file_watcher_file_changed_receiver: std::sync::mpsc::Receiver<notify::DebouncedEvent>,
}

struct ProjectData {
    data: Project,
    undoer: Undoer<Project>,
}

impl Default for RootState {
    fn default() -> Self {
        let (open_image_request_sender, open_image_request_receiver) = flume::bounded(1);
        let (open_image_response_sender, open_image_response_receiver) = flume::bounded(1);

        // Spawn the file dialog thread
        std::thread::spawn(move || {
            while let Ok(name) = open_image_request_receiver.recv() {
                trc::trace!("Got request for file load: {}", name);
                let file = FileDialog::new()
                    .set_location("~/Desktop")
                    .add_filter("PNG Image", &["png"])
                    .show_open_single_file()
                    .expect("Show file dialog");

                if let Some(path) = file {
                    open_image_response_sender.send((name, path)).ok();
                } else {
                    trc::trace!("No file picked");
                }
            }
        });

        let (file_watcher_path_change_sender, file_watcher_path_change_receiver) =
            std::sync::mpsc::channel();
        let (file_watcher_file_change_sender, file_watcher_file_changed_receiver) =
            std::sync::mpsc::channel();

        // Spawn the file watcher thread
        std::thread::spawn(move || {
            // This is used to keep the watcher in scope while it listens for changes
            let mut watcher: Option<notify::RecommendedWatcher> = None;
            let mut prev_path = None;

            while let Ok(path) = file_watcher_path_change_receiver.recv() {
                if let Some(mut watcher) = watcher.take() {
                    if let Some(prev_path) = prev_path.take() {
                        watcher.unwatch(prev_path).expect("Failed to unwatch file");
                    }
                }

                let mut new_watcher = notify::watcher(
                    file_watcher_file_change_sender.clone(),
                    Duration::from_secs(1),
                )
                .expect("Start file watcher");

                new_watcher
                    .watch(&path, notify::RecursiveMode::NonRecursive)
                    .expect("Watch filesystem");

                prev_path = Some(path);
                watcher = Some(new_watcher);
            }
        });

        Self {
            project: None,
            start: Instant::now(),
            open_image_request_sender,
            open_image_response_receiver,
            file_watcher_path_change_sender,
            file_watcher_file_changed_receiver,
        }
    }
}

impl NesimgGui {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Scale up the UI slightly
        cc.egui_ctx.set_pixels_per_point(1.2);
        // Scale down the feathering slightly to compensate and keep edges from looking a little
        // blurry
        cc.egui_ctx.tessellation_options().feathering_size_in_pixels = 0.7;

        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            let gui: NesimgGui = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();

            if gui.dark_mode {
                cc.egui_ctx.set_visuals(egui::style::Visuals::dark());
            }

            gui
        } else {
            // Default to dark theme
            cc.egui_ctx.set_visuals(egui::style::Visuals::dark());

            Default::default()
        }
    }

    fn toggle_dark_mode(&mut self, ui: &mut Ui) {
        if ui.visuals().dark_mode {
            self.dark_mode = false;
            ui.ctx().set_visuals(egui::Visuals::light())
        } else {
            self.dark_mode = true;
            ui.ctx().set_visuals(egui::Visuals::dark())
        }
    }
}

/// Actions that can be triggered by menus or keyboard shortcuts
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) enum MainGuiAction {
    Quit,
    LoadProject,
    SaveProject,
    Undo,
}

impl MainGuiAction {
    fn perform(&self, gui: &mut NesimgGui, ctx: &egui::Context, frame: &mut eframe::Frame) {
        #[allow(clippy::unit_arg)]
        if let Err(e) = match self {
            MainGuiAction::Quit => Ok(frame.quit()),
            MainGuiAction::LoadProject => Ok(gui
                .state
                .open_image_request_sender
                .send("project")
                .expect("Open file")),
            MainGuiAction::SaveProject => save_project(gui, ctx),
            MainGuiAction::Undo => {
                if let Some(project) = &mut gui.state.project {
                    if let Some(undone) = project.undoer.undo(&project.data) {
                        project.data = undone.clone();
                    }
                }

                Ok(())
            }
        } {
            trc::error!("{}", e);
            send_error_notification(ctx, e.to_string());
        }
    }
}

/// Keyboard shortcuts that can trigger [`MainGuiAction`]s
static MAIN_GUI_SHORTCUTS: Lazy<HashMap<MainGuiAction, KeyboardShortcut>> = Lazy::new(|| {
    let mut shortcuts = HashMap::default();

    shortcuts.insert(MainGuiAction::Quit, (Modifiers::COMMAND, Key::Q).into());
    shortcuts.insert(
        MainGuiAction::LoadProject,
        (Modifiers::COMMAND, Key::O).into(),
    );
    shortcuts.insert(
        MainGuiAction::SaveProject,
        (Modifiers::COMMAND, Key::S).into(),
    );
    shortcuts.insert(MainGuiAction::Undo, (Modifiers::COMMAND, Key::Z).into());

    shortcuts
});

/// GUI implementation
impl eframe::App for NesimgGui {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        handle_keyboard_shortcuts(self, ctx, frame);
        handle_file_loads(self, ctx);

        show_notifications(ctx);

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.add_space(1.0);
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    let open_shortcut = MAIN_GUI_SHORTCUTS
                        .get(&MainGuiAction::LoadProject)
                        .map_or(String::new(), |x| format!("\t{}", x));
                    let save_shortcut = MAIN_GUI_SHORTCUTS
                        .get(&MainGuiAction::SaveProject)
                        .map_or(String::new(), |x| format!("\t{}", x));
                    let quit_shortcut = MAIN_GUI_SHORTCUTS
                        .get(&MainGuiAction::Quit)
                        .map_or(String::new(), |x| format!("\t{}", x));

                    if ui
                        .button(format!("Open Project{}", open_shortcut))
                        .clicked()
                    {
                        MainGuiAction::SaveProject.perform(self, ctx, frame);
                        ui.close_menu();
                    }

                    ui.add_enabled_ui(self.state.project.is_some(), |ui| {
                        if ui
                            .button(format!("Save Project{}", save_shortcut))
                            .clicked()
                        {
                            MainGuiAction::SaveProject.perform(self, ctx, frame);
                            ui.close_menu();
                        }
                    });

                    ui.separator();

                    if ui.button(format!("Quit{}", quit_shortcut)).clicked() {
                        frame.quit();
                    }
                });

                ui.menu_button("Edit", |ui| {
                    ui.add_enabled_ui(self.state.project.is_some(), |ui| {
                        let undo_shortcut = MAIN_GUI_SHORTCUTS
                            .get(&MainGuiAction::Undo)
                            .map_or(String::new(), |x| format!("\t{}", x));

                        if ui.button(format!("Undo {}", undo_shortcut)).clicked() {
                            MainGuiAction::Undo.perform(self, ctx, frame);
                        }
                    });
                });

                ui.menu_button("UI", |ui| {
                    if ui.checkbox(&mut self.dark_mode, "Dark Theme").clicked() {
                        self.toggle_dark_mode(ui);
                    }
                });

                // Tab list
                let tabs = ui.with_layout(Layout::right_to_left(), |ui| {
                    if self.state.project.is_none() {
                        ui.set_enabled(false);
                    }
                    ui.horizontal(|ui| {
                        for (name, _) in &self.tabs {
                            ui.selectable_value(&mut self.current_tab, name.clone(), name);
                        }
                    });
                    ui.separator();
                });
                if self.state.project.is_none() {
                    tabs.response
                        .on_hover_text_at_pointer("Open project to edit");
                }
            });
        });

        // Render the actual tab contents
        if self.state.project.is_some() {
            for (name, tab) in &mut self.tabs {
                if name == &self.current_tab {
                    tab.show(&mut self.state, ctx, frame);
                }
            }
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.set_width(ui.available_height() / 3.0);
                    ui.set_height(ui.available_width() / 3.0);
                    if ui.button("Open Project").clicked() {
                        MainGuiAction::LoadProject.perform(self, ctx, frame);
                    }
                });
            });
        }

        // Update the undo state for the project, if one has been loaded
        if let Some(project) = &mut self.state.project {
            project
                .undoer
                .feed_state(self.state.start.elapsed().as_secs_f64(), &project.data);
        }
    }
}

fn handle_keyboard_shortcuts(gui: &mut NesimgGui, ctx: &egui::Context, frame: &mut eframe::Frame) {
    for (action, shortcut) in &*MAIN_GUI_SHORTCUTS {
        if ctx
            .input_mut()
            .consume_key(shortcut.modifiers, shortcut.key)
        {
            action.perform(gui, ctx, frame);
        }
    }
}

fn handle_file_loads(gui: &mut NesimgGui, ctx: &egui::Context) {
    // Load the source image if the user has selected one
    if let Ok((name, path)) = gui.state.open_image_response_receiver.try_recv() {
        match name {
            _ => panic!("Unrecognized file loaded"),
        }
    }
}

fn save_project(gui: &mut NesimgGui, ctx: &egui::Context) -> anyhow::Result<()> {
    todo!()
}
