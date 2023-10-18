use core::arch::global_asm;

extern "C" {
    pub static mut __boot_start: u8;
}

global_asm!(
    include_str!("start.s"),
    MPIDR_CPU_ID_MASK = const 0b11,
);
