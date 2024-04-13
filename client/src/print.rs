use core::fmt::{self, Write};

use crate::bsp::serial;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::print::_print(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print::_print(format_args_nl!(""));
    };
    ($($arg:tt)*) => {{
        $crate::print::_print(format_args_nl!($($arg)*));
    }};
}

pub fn _print(args: fmt::Arguments) {
    let mut writer = Writer {};
    let _ = writer.write_fmt(args);
}

struct Writer {}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        serial::write_bytes(s.as_bytes());
        Ok(())
    }
}
