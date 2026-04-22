/*++

Licensed under the Apache-2.0 license.

File Name:

    main.rs

Abstract:

    File contains main entry point for Network Coprocessor ROM.
    When no test feature is enabled, runs default firmware (banner + WFI loop).
    Test features gate specific test entry points.

--*/

#![cfg_attr(target_arch = "riscv32", no_std)]
#![no_main]

#[cfg(target_arch = "riscv32")]
use core::panic::PanicInfo;

#[cfg(target_arch = "riscv32")]
use core::arch::global_asm;

use network_config::DEFAULT_NETWORK_MEMORY_MAP;

#[cfg(any(
    feature = "test-hello-world",
    feature = "test-dccm",
    feature = "test-exception"
))]
mod tests;

// Include the startup assembly code
#[cfg(target_arch = "riscv32")]
global_asm!(include_str!("start.s"));

/// UART TX data register address for Network Coprocessor
/// This is UART offset + TX register offset (0x41)
const UART_TX_ADDR: u32 = DEFAULT_NETWORK_MEMORY_MAP.uart_offset + 0x41;

/// Emulator control register for exit
const EMU_CTRL_EXIT: u32 = DEFAULT_NETWORK_MEMORY_MAP.ctrl_offset;

/// MCI debug output register — used for VCS testbench printf and exit signaling.
/// MCU uses this same register; NWP can reach it via AXI slave S4 (MCI).
const MCI_DEBUG_OUT: u32 = 0x2100_0414;

/// Print a single character to the UART
#[inline(never)]
fn print_char(c: u8) {
    unsafe {
        core::ptr::write_volatile(UART_TX_ADDR as *mut u8, c);
    }
}

/// Print a string to the UART
fn print_str(s: &str) {
    for b in s.bytes() {
        print_char(b);
    }
}

/// Print a u32 value as hex to the UART
#[allow(dead_code)]
fn print_hex(val: u32) {
    const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";
    print_str("0x");
    for i in (0..8).rev() {
        let nibble = ((val >> (i * 4)) & 0xF) as usize;
        print_char(HEX_DIGITS[nibble]);
    }
}

/// Write a byte to MCI debug output register (for VCS testbench signaling).
/// In emulator, this write goes to emulated MCI (harmless).
/// In VCS, testbench monitors this for TB_CMD_END_SIM_WITH_SUCCESS (0xFF) / FAILURE (0x01).
fn exit_vcs(code: u8) {
    unsafe {
        core::ptr::write_volatile(MCI_DEBUG_OUT as *mut u32, code as u32);
    }
}

/// Exit the emulator with the given code.
/// In VCS, this write goes to NC0 slave (harmless no-op).
fn exit_emulator(code: u32) -> ! {
    unsafe {
        core::ptr::write_volatile(EMU_CTRL_EXIT as *mut u32, code);
    }
    #[allow(clippy::empty_loop)]
    loop {
        #[cfg(target_arch = "riscv32")]
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}

/// Main entry point called from assembly startup code
#[cfg(target_arch = "riscv32")]
#[no_mangle]
pub extern "C" fn main() -> ! {
    // Print hello world message
    print_str("\n");
    print_str("=====================================\n");
    print_str("  Network Coprocessor ROM Started!  \n");
    print_str("=====================================\n");

    // Feature-gated test dispatch (first matching feature wins)
    #[cfg(feature = "test-hello-world")]
    tests::hello_world::run(print_str, exit_emulator, exit_vcs);

    #[cfg(all(feature = "test-dccm", not(feature = "test-hello-world")))]
    tests::dccm::run(print_str, print_hex, exit_emulator, exit_vcs);

    #[cfg(all(
        feature = "test-exception",
        not(feature = "test-hello-world"),
        not(feature = "test-dccm")
    ))]
    tests::exception::run(print_str, exit_emulator, exit_vcs);

    // Default: no test feature — just loop
    #[cfg(not(any(
        feature = "test-hello-world",
        feature = "test-dccm",
        feature = "test-exception"
    )))]
    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}

/// Exception handler — called when CPU encounters an exception.
/// Matches MCU ROM pattern: reads diagnostics, prints, and exits (never returns).
///
/// For `test-exception` feature: prints PASS and exits with success (verifies handler fired).
/// For all other modes: prints error and exits with failure.
#[no_mangle]
pub extern "C" fn exception_handler() -> ! {
    // Read diagnostic CSRs (same as MCU ROM exception_handler)
    let mcause: u32;
    let mepc: u32;
    unsafe {
        core::arch::asm!(
            "csrr {mcause}, mcause",
            "csrr {mepc}, mepc",
            mcause = out(reg) mcause,
            mepc = out(reg) mepc,
        );
    }

    #[cfg(feature = "test-exception")]
    {
        // Exception handler fired — that's what we're testing
        print_str("  mcause=");
        print_hex(mcause);
        print_str(" mepc=");
        print_hex(mepc);
        print_str("\n");
        print_str("NWP exception test PASS\n");
        exit_vcs(0xFF);
        exit_emulator(0x00);
    }

    #[cfg(not(feature = "test-exception"))]
    {
        print_str("EXCEPTION: mcause=");
        print_hex(mcause);
        print_str(" mepc=");
        print_hex(mepc);
        print_str("\n");
        print_str("EXCEPTION: Network ROM encountered an error!\n");
        exit_vcs(0x01);
        exit_emulator(0x01);
    }
}

/// Panic handler for no_std environment
#[cfg(target_arch = "riscv32")]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print_str("PANIC: Network ROM panicked!\n");
    exit_vcs(0x01);
    exit_emulator(0x01);
}

// Dummy main for non-RISC-V targets (for cargo check on host)
#[cfg(not(target_arch = "riscv32"))]
#[no_mangle]
pub extern "C" fn main() {
    println!("Network ROM (host build - no-op)");
}
