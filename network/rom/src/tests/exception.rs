// Licensed under the Apache-2.0 license

//! NWP Exception test — validates exception handler fires on illegal instruction.
//!
//! Matches MCU ROM exception handler pattern: the handler catches the exception,
//! prints diagnostic info, and exits. It does NOT resume execution (-> !).
//!
//! Flow:
//! 1. Firmware triggers `unimp` (illegal instruction)
//! 2. Exception handler reads mcause/mepc, prints PASS, exits
//! 3. Integration test checks for "NWP exception test PASS" in UART

/// Run the exception handler test.
///
/// Triggers an illegal instruction. If the exception handler works, it will
/// print PASS and exit before this function returns. If the handler does NOT
/// fire, execution falls through to the FAIL path.
pub fn run(
    print_str: fn(&str),
    exit_emulator: fn(u32) -> !,
    exit_vcs: fn(u8),
) -> ! {
    print_str("NWP test: exception\n");
    print_str("  Triggering illegal instruction...\n");

    // Execute an illegal instruction (same as MCU's `unimp`)
    // The exception handler should catch this and exit with PASS
    unsafe {
        core::arch::asm!("unimp");
    }

    // If we reach here, exception handler did NOT fire
    print_str("NWP exception test FAIL: handler did not run\n");
    exit_vcs(0x01);
    exit_emulator(0x01);
}
