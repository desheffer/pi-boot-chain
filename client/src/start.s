.section ".text._start"

.globl _start
_start:
    // Get the relocation addresses.
    // x9 = cursor to old address (mutable)
    // x10 = cursor to new address (mutable)
    // x11 = new end address
    ldr x9, =__boot_start
    ldr x10, =__start
    ldr x11, =__end

    // Relocate.
loop_relocate:
    ldp x12, x13, [x9], #16
    stp x12, x13, [x10], #16
    cmp x10, x11
    blo loop_relocate

    // Get the range of BSS.
    ldr x9, =__bss_start
    ldr x10, =__bss_end

    // Zero out BSS.
loop_clear_bss:
    stp xzr, xzr, [x9], #16
    cmp x9, x10
    blo loop_clear_bss

    // Set up initial stack pointer.
    ldr x9, =__stack_start
    mov sp, x9

    // Jump to kernel main.
    dsb sy
    isb sy
    ldr x9, =kernel_main
    blr x9

hang:
    wfe
    b hang

.globl jump
jump:
    // Jump to new kernel.
    dsb sy
    isb sy
    ldr x9, =__boot_start
    br x9
