use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::os::unix::io::AsRawFd;
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSADRAIN, VMIN, VTIME};

pub const DEFAULT_PATH: &str = "/dev/tty";

/// Provides I/O for a terminal.
pub struct TtyAdapter {
    tty: File,
}

impl TtyAdapter {
    pub fn new(path: String) -> io::Result<Self> {
        let tty = OpenOptions::new()
            .write(true)
            .read(true)
            .open(path.to_owned())?;
        let fd = tty.as_raw_fd();
        let mut termios = Termios::from_fd(fd)?;
        termios.c_lflag &= !(ECHO | ICANON);
        termios.c_cc[VMIN] = 0;
        termios.c_cc[VTIME] = 0;
        tcsetattr(fd, TCSADRAIN, &termios)?;

        Ok(Self { tty })
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        let mut buf: Vec<u8> = vec![0; 1];
        match self.tty.read(buf.as_mut_slice()) {
            Err(_) => None,
            Ok(0) => None,
            Ok(_) => Some(buf[0]),
        }
    }
}
