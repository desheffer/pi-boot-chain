use serialport::{self, SerialPort};
use std::io::{self, Read, Write};

/// Provides I/O for a serial port.
pub struct SerialAdapter {
    serial: Box<dyn SerialPort>,
}

impl SerialAdapter {
    pub fn new(path: String) -> io::Result<Self> {
        let serial = serialport::new(path.to_owned(), 115_200).open()?;
        Ok(Self { serial })
    }

    pub fn name(&self) -> String {
        self.serial.name().unwrap_or("serial".to_owned())
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        let mut buf: Vec<u8> = vec![0; 1];
        match self.serial.read(buf.as_mut_slice()) {
            Err(_) => None,
            Ok(0) => None,
            Ok(_) => Some(buf[0]),
        }
    }

    pub fn write_byte(&mut self, byte: u8) -> io::Result<()> {
        let buf: Vec<u8> = vec![byte];
        self.serial.write_all(buf.as_slice())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.serial.flush()
    }
}
