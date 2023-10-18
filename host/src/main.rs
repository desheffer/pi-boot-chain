#[path = "../../common/mod.rs"]
mod common;

mod event;
mod serial;
mod state;
mod tty;

use clap::Parser;
use colored::*;
use std::process;

use crate::event::StdoutEventHandler;
use crate::serial::SerialAdapter;
use crate::state::StateMachine;
use crate::tty::TtyAdapter;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to serial device
    #[arg(index(1))]
    serial: String,

    /// Path to boot image
    #[arg(index(2))]
    image: String,
}

fn main() {
    let args = Args::parse();

    if let Err(error) = serve(args.serial, tty::DEFAULT_PATH.to_owned(), args.image) {
        eprintln!("{}: {}", "Error".bold().red(), error);
        process::exit(1);
    }
}

fn serve(serial_path: String, tty_path: String, image_path: String) -> Result<(), String> {
    println!(
        "{} {} is {}",
        "🚀",
        "Pi Boot Chain".bold().cyan(),
        "Ready".bold().green()
    );

    let serial = SerialAdapter::new(serial_path.to_owned())
        .map_err(|e| format!("{}: {}", serial_path, e.to_string()))?;

    let tty = TtyAdapter::new(tty_path.to_owned())
        .map_err(|e| format!("{}: {}", tty_path, e.to_string()))?;

    let mut event_handler = StdoutEventHandler::new();

    StateMachine::new(serial, tty, image_path, &mut event_handler).event_loop()
}
