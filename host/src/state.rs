use std::fs::File;
use std::io::{Bytes, Read};
use std::primitive;
use std::thread;
use std::time::Duration;

use crate::common::{HEADER_PREAMBLE, OK_PAYLOAD, RESET_PAYLOAD};
use crate::serial::SerialAdapter;
use crate::tty::TtyAdapter;

/// Represents a state to be managed by the state machine.
pub enum State {
    Terminal(usize),
    LoadingImage,
    SendingHeader(usize),
    WaitingForOk(usize),
    SendingImage(u64),
}

/// Represents events that can be emitted by the state machine.
pub enum Event {
    TerminalOutput(u8),
    TransferProgress(u64, u64),
}

/// Handles events emitted by the state machine.
pub trait EventHandler {
    fn handle(&mut self, event: Event);
}

/// Represents serial port interactions as a state machine.
pub struct StateMachine<'a> {
    serial: SerialAdapter,
    tty: TtyAdapter,
    image_path: String,
    state: State,
    header_payload: Vec<u8>,
    image_size: u64,
    image_payload: Option<Bytes<File>>,
    event_handler: &'a mut dyn EventHandler,
}

impl<'a> StateMachine<'a> {
    pub fn new(
        serial: SerialAdapter,
        tty: TtyAdapter,
        image_path: String,
        event_handler: &'a mut dyn EventHandler,
    ) -> Self {
        Self {
            serial,
            tty,
            image_path,
            state: State::Terminal(0),
            header_payload: Vec::new(),
            image_size: 0,
            image_payload: None,
            event_handler,
        }
    }

    /// Runs the event loop.
    ///
    /// This delegates to other functions based on the current state, then performs a state update
    /// based on the output of those functions.
    pub fn event_loop(&mut self) -> Result<(), String> {
        loop {
            self.state = match self.state {
                State::Terminal(progress) => self.do_terminal(progress)?,
                State::LoadingImage => self.do_load_image()?,
                State::SendingHeader(progress) => self.do_send_header(progress)?,
                State::WaitingForOk(progress) => self.do_wait_for_ok(progress)?,
                State::SendingImage(progress) => self.do_send_image(progress)?,
            };
        }
    }

    /// Handles terminal mode.
    ///
    /// This mode connects stdin and stdout with the serial device. Anything the user types is
    /// written to the serial device, and any output from the serial device is printed to the
    /// screen.
    ///
    /// This mode advances when the `RESET_PAYLOAD` is received.
    fn do_terminal(&mut self, progress: usize) -> Result<State, String> {
        let expected_byte = RESET_PAYLOAD[progress];

        if let Some(byte) = self.tty.read_byte() {
            self.serial
                .write_byte(byte)
                .map_err(|e| format!("{}: {}", self.serial.name(), e.to_string()))?;
        }

        match self.serial.read_byte() {
            // No change.
            None => {
                thread::sleep(Duration::from_millis(10));
                Ok(State::Terminal(progress))
            }
            // Got expected byte from reset sequence.
            Some(byte) if byte == expected_byte => {
                let progress = progress + 1;
                if progress < RESET_PAYLOAD.len() {
                    // There are more bytes in the sequence. Increment progress counter.
                    Ok(State::Terminal(progress))
                } else {
                    // Received all bytes in the sequence. Go to next step.
                    Ok(State::LoadingImage)
                }
            }
            // Got regular byte. Print and reset.
            Some(byte) => {
                self.event_handler.handle(Event::TerminalOutput(byte));
                Ok(State::Terminal(0))
            }
        }
    }

    /// Loads the boot image.
    ///
    /// The boot image is always reloaded from disk so that the most up-to-date version is sent.
    ///
    /// This mode advances if the image is loaded successfully.
    fn do_load_image(&mut self) -> Result<State, String> {
        let image = File::open(&self.image_path)
            .map_err(|e| format!("{}: {}", self.image_path, e.to_string()))?;

        self.image_size = image
            .metadata()
            .map_err(|e| format!("{}: {}", self.image_path, e.to_string()))?
            .len();

        self.event_handler
            .handle(Event::TransferProgress(0, self.image_size));

        self.header_payload.clear();
        self.header_payload.extend(HEADER_PREAMBLE);
        for byte in primitive::u32::to_le_bytes(self.image_size as u32) {
            self.header_payload.push(byte);
        }

        self.image_payload = Some(image.bytes());

        // Loaded image. Go to next step.
        Ok(State::SendingHeader(0))
    }

    /// Sends the header.
    ///
    /// The header consists of a preamble and the image size.
    ///
    /// This mode advances after all bytes are written.
    fn do_send_header(&mut self, progress: usize) -> Result<State, String> {
        self.serial
            .write_byte(self.header_payload[progress])
            .map_err(|e| format!("{}: {}", self.serial.name(), e.to_string()))?;

        self.serial
            .flush()
            .map_err(|e| format!("{}: {}", self.serial.name(), e.to_string()))?;

        let progress = progress + 1;
        if progress < self.header_payload.len() {
            // There are more bytes to send. Increment progress counter.
            Ok(State::SendingHeader(progress))
        } else {
            // Header sent. Go to next step.
            Ok(State::WaitingForOk(0))
        }
    }

    /// Waits for acknowledgment of the header.
    ///
    /// This mode advances when the `OK_PAYLOAD` is received.
    fn do_wait_for_ok(&mut self, progress: usize) -> Result<State, String> {
        let expected_byte = OK_PAYLOAD[progress];
        match self.serial.read_byte() {
            // No change.
            None => Ok(State::WaitingForOk(progress)),
            // Got expected byte from OK sequence.
            Some(byte) if byte == expected_byte => {
                let progress = progress + 1;
                if progress < OK_PAYLOAD.len() {
                    // There are more bytes in the sequence. Increment progress counter.
                    Ok(State::WaitingForOk(progress))
                } else {
                    // Received all bytes in the sequence. Go to next step.
                    Ok(State::SendingImage(0))
                }
            }
            // Got unexpected byte. Reset.
            Some(_) => Ok(State::Terminal(0)),
        }
    }

    /// Sends the image.
    ///
    /// This is the main event.
    ///
    /// This mode advances after all bytes are written. At this point, the application is reset to
    /// terminal mode to allow interaction with the serial device.
    fn do_send_image(&mut self, progress: u64) -> Result<State, String> {
        self.event_handler
            .handle(Event::TransferProgress(progress, self.image_size));

        let payload = self.image_payload.as_mut().unwrap();
        let next_byte = payload
            .next()
            .unwrap()
            .map_err(|e| format!("{}: {}", self.image_path, e.to_string()))?;

        self.serial
            .write_byte(next_byte)
            .map_err(|e| format!("{}: {}", self.serial.name(), e.to_string()))?;

        self.serial
            .flush()
            .map_err(|e| format!("{}: {}", self.serial.name(), e.to_string()))?;

        let progress = progress + 1;
        if progress < self.image_size {
            // There are more bytes to send. Increment progress counter.
            Ok(State::SendingImage(progress))
        } else {
            self.event_handler
                .handle(Event::TransferProgress(progress, self.image_size));

            // Image sent. Go to terminal mode.
            Ok(State::Terminal(0))
        }
    }
}
