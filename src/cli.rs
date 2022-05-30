use std::path::PathBuf;

use structopt::StructOpt;
use tracing as trc;

#[derive(Debug, structopt::StructOpt)]
#[structopt(
    name = "NESImg",
    about = "A background, sprite, and map editor for making NES games."
)]
enum Args {
    #[structopt(about = "Start the GUI interface")]
    Gui(GuiArgs),
}

#[derive(Debug, structopt::StructOpt)]
pub struct GuiArgs {
    pub project: Option<PathBuf>,
}

pub fn run() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    setup_tracing();

    let args = Args::from_args();
    trc::debug!(?args, "Parsed commandline arguments");

    match args {
        Args::Gui(args) => crate::gui::run_gui(args),
    }
}

fn setup_tracing() {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer();
    let filter_layer =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("nesimg_gui=debug"));

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
}
