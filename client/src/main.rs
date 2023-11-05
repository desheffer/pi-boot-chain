#![feature(asm_const)]
#![feature(format_args_nl)]
#![no_std]
#![no_main]

#[path = "../../common/mod.rs"]
mod common;

mod bsp;
mod start;

use core::panic::PanicInfo;
use core::primitive;

use crate::bsp::serial;
use crate::common::{HEADER_PREAMBLE, OK_PAYLOAD, RESET_PAYLOAD};
use crate::start::{__boot_start, jump};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_main(x0: u64, x1: u64, x2: u64, x3: u64) {
    serial::init();
    serial::write_bytes(b"\n*** system booting ***\n");

    serial::write_bytes(&RESET_PAYLOAD);

    let mut verify = 0;
    while verify < HEADER_PREAMBLE.len() {
        verify = match serial::read_byte() {
            byte if byte == HEADER_PREAMBLE[verify] => verify + 1,
            byte if byte == HEADER_PREAMBLE[0] => 1,
            _ => 0,
        };
    }

    let mut size_buffer = [0; 4];
    for i in 0..size_buffer.len() {
        size_buffer[i] = serial::read_byte();
    }
    let size = primitive::u32::from_le_bytes(size_buffer);

    serial::write_bytes(&OK_PAYLOAD);

    unsafe {
        copy(size as usize);
        jump(x0, x1, x2, x3);
    }
}

pub unsafe fn copy(size: usize) {
    let boot_start = &mut __boot_start as *mut u8;
    for i in 0..size {
        boot_start.add(i).write_volatile(serial::read_byte());
    }
}
