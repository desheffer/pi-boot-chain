use core::arch::global_asm;

extern "C" {
    pub static mut __boot_start: u8;

    pub fn jump();
}

global_asm!(include_str!("start.s"));
