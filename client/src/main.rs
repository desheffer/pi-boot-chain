#![feature(asm_const)]
#![feature(format_args_nl)]
#![no_std]
#![no_main]

#[path = "../../common/mod.rs"]
mod common;

mod bsp;
mod start;

use core::arch::asm;
use core::panic::PanicInfo;
use core::primitive;

use crate::bsp::serial;
use crate::common::{HEADER_PREAMBLE, OK_PAYLOAD, RESET_PAYLOAD};
use crate::start::__boot_start;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_main() {
    serial::init();
    serial::write_bytes(b"\n*** system booting ***\n");

    serial::write_bytes(&RESET_PAYLOAD);

    loop {
        for i in 0..HEADER_PREAMBLE.len() {
            if serial::read_byte() != HEADER_PREAMBLE[i] {
                break;
            }
        }
        break;
    }

    let mut size_buffer = [0; 4];
    for i in 0..size_buffer.len() {
        size_buffer[i] = serial::read_byte();
    }
    let size = primitive::u32::from_le_bytes(size_buffer);

    serial::write_bytes(&OK_PAYLOAD);

    copy(size as usize);
}

pub fn copy(size: usize) {
    unsafe {
        let boot_start = &mut __boot_start as *mut u8;

        for i in 0..size {
            boot_start.add(i).write_volatile(serial::read_byte());
        }

        asm!("br {}", in(reg) boot_start);
    }
}
