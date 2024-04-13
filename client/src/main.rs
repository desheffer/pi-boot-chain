#![feature(allocator_api)]
#![feature(asm_const)]
#![feature(format_args_nl)]
#![feature(panic_info_message)]
#![no_std]
#![no_main]

use core::primitive::u32;

use alloc::vec::Vec;
use miniz_oxide::inflate::decompress_to_vec;

use crate::bsp::serial;
use crate::common::{HEADER_PREAMBLE, OK_PAYLOAD, RESET_PAYLOAD};
use crate::start::{__boot_start, jump};

extern crate alloc;

mod bsp;
mod common;
mod heap;
mod panic;
mod print;
mod start;

#[no_mangle]
unsafe extern "C" fn kernel_main(x0: u64, x1: u64, x2: u64, x3: u64) {
    serial::init();

    println!();
    println!("*** system booting ***");

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
    let size = u32::from_le_bytes(size_buffer);

    serial::write_bytes(&OK_PAYLOAD);

    let mut compressed_image: Vec<u8> = Vec::with_capacity(size as _);
    for _ in 0..size {
        compressed_image.push(serial::read_byte());
    }

    let image = decompress_to_vec(&compressed_image).unwrap();

    let boot_start = &mut __boot_start as *mut u8;
    for i in 0..image.len() {
        boot_start.add(i).write_volatile(image[i]);
    }

    jump(x0, x1, x2, x3);
}
