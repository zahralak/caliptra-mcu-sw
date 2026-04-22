/*++

Licensed under the Apache-2.0 license.

File Name:

    start.s

Abstract:

    File contains startup code for Network Coprocessor ROM (bare-metal RISCV)

--*/

.option norvc
.option norelax

# Constants (assembler equates — no bytes emitted)
.equ  EMU_CTRL_EXIT, 0x10002000

.section .text.init
.global _start
_start:

    la gp, GLOBAL_POINTER

    # Initialize the stack pointer
    la sp, STACK_TOP

    # Configure mtvec with the address of the exception handler in direct mode
    la t0, _exception_handler
    csrw mtvec, t0

    # Initialize MRAC (Region Access Control Register)
    # CSR address 0x7c0 = MRAC register
    lui     t0, %hi(MRAC_VALUE)
    addi    t0, t0, %lo(MRAC_VALUE)
    csrw    0x7c0, t0

    # Clear BSS section
    la t0, BSS_START
    la t1, BSS_END
clear_bss:
    bge t0, t1, end_clear_bss
    sw x0, 0(t0)
    addi t0, t0, 4
    j clear_bss
end_clear_bss:

    # Copy initialized data from ROM to RAM
    la t0, ROM_DATA_START
    la t1, DATA_START
    la t2, DATA_END
copy_data:
    bge t1, t2, end_copy_data
    lw t3, 0(t0)
    sw t3, 0(t1)
    addi t0, t0, 4
    addi t1, t1, 4
    j copy_data
end_copy_data:

    # Call main entry point
    call main

    # If main returns, exit the emulator
    li t0, EMU_CTRL_EXIT
    sw zero, 0(t0)

    # Infinite loop (should never reach here)
1:  j 1b

# Exception handler entry point — must be 4-byte aligned for mtvec direct mode.
# mtvec bits [1:0] are the mode field; the handler address must have bits [1:0] = 0.
.p2align 2
_exception_handler:
    # Save the SP to mscratch
    csrw mscratch, sp

    # Use a simple exception stack (reuse main stack for simplicity)
    la sp, STACK_TOP
    addi sp, sp, -64

    # Call the exception handler function (never returns — matches MCU ROM pattern)
    jal exception_handler

    # Unreachable: exception_handler is -> ! (divergent)
    # If it somehow returns, loop forever
1:  j 1b
