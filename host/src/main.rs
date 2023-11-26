use clap::Parser;
use colored::*;
use std::thread;
use std::time::Duration;

use crate::event::StdoutEventHandler;
use crate::serial::SerialAdapter;
use crate::state::StateMachine;
use crate::tty::TtyAdapter;

mod event;
mod serial;
mod state;
mod tty;

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

    println!(
        "{} {} is {}",
        "🚀",
        "Pi Boot Chain".bold().cyan(),
        "Ready".bold().green()
    );

    loop {
        match serve(
            args.serial.to_owned(),
            tty::DEFAULT_PATH.to_owned(),
            args.image.to_owned(),
        ) {
            Ok(_) => break,
            Err(error) => {
                eprintln!("{}: {}", "Error".bold().red(), error);
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

fn serve(serial_path: String, tty_path: String, image_path: String) -> Result<(), String> {
    let serial = SerialAdapter::new(serial_path.to_owned())
        .map_err(|e| format!("{}: {}", serial_path, e.to_string()))?;

    let tty = TtyAdapter::new(tty_path.to_owned())
        .map_err(|e| format!("{}: {}", tty_path, e.to_string()))?;

    let mut event_handler = StdoutEventHandler::new();

    StateMachine::new(serial, tty, image_path, &mut event_handler).event_loop()
}
