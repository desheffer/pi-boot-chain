use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, Write};

use crate::state::{Event, EventHandler};

/// Displays information about events emitted by the state machine.
pub struct StdoutEventHandler {
    progress_bar: Option<ProgressBar>,
}

impl StdoutEventHandler {
    pub fn new() -> Self {
        Self { progress_bar: None }
    }
}

impl EventHandler for StdoutEventHandler {
    fn handle(&mut self, event: Event) {
        match event {
            Event::TerminalOutput(byte) => {
                print!("{}", byte as char);
                io::stdout().flush().unwrap();
            }
            Event::TransferProgress(sent, total) => {
                if let None = self.progress_bar {
                    let pb = ProgressBar::new(total);
                    pb.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                        .unwrap());
                    self.progress_bar = Some(pb);
                }
                self.progress_bar.as_mut().unwrap().set_position(sent);
                if sent == total {
                    self.progress_bar.as_mut().unwrap().finish();
                    self.progress_bar = None;
                }
            }
        };
    }
}
