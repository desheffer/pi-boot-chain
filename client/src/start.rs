use core::arch::global_asm;

extern "C" {
    pub static mut __boot_start: u8;

    pub fn jump(x0: u64, x1: u64, x2: u64, x3: u64);
}

global_asm!(include_str!("start.s"));
