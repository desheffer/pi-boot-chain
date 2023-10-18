.section ".text._start"

.globl _start
_start:
    // Isolate core 0.
    mrs x0, mpidr_el1
    and x0, x0, {MPIDR_CPU_ID_MASK}
    cbnz x0, hang

    // Get the relocation addresses.
    // x0 = cursor to old address (mutable)
    // x1 = cursor to new address (mutable)
    // x2 = new end address
    ldr x0, =__boot_start
    ldr x1, =__start
    ldr x2, =__end

    // Relocate.
loop_relocate:
    ldp x4, x5, [x0], #16
    stp x4, x5, [x1], #16
    cmp x1, x2
    blo loop_relocate

    // Get the range of BSS.
    ldr x0, =__bss_start
    ldr x1, =__bss_end

    // Zero out BSS.
loop_clear_bss:
    stp xzr, xzr, [x0], #16
    cmp x0, x1
    blo loop_clear_bss

    // Set up initial stack pointer.
    ldr x0, =__stack_start
    mov sp, x0

    ldr x0, =kernel_main
    blr x0

hang:
    wfe
    b hang
