// Licensed under the Apache-2.0 license

//! NWP Hello World test — validates basic boot and DCCM access.
//!
//! Writes a known pattern to DCCM (past .bss), reads it back, and verifies.

/// Test pattern to write/read from DCCM
const TEST_PATTERN: u32 = 0xDEAD_BEEF;

extern "C" {
    static BSS_END: u32;
}

/// Run the hello world test.
///
/// Uses BSS_END linker symbol to find safe DCCM region past .data/.bss sections.
pub fn run(
    print_str: fn(&str),
    exit_emulator: fn(u32) -> !,
    exit_vcs: fn(u8),
) -> ! {
    print_str("NWP test: hello_world\n");

    // Use address past BSS_END (aligned to 4 bytes) to avoid .data/.bss corruption
    let test_addr = core::ptr::addr_of!(BSS_END) as u32;
    // Align up to 4 bytes
    let test_addr = (test_addr + 3) & !3;
    let test_ptr = test_addr as *mut u32;

    // Write test pattern
    unsafe {
        core::ptr::write_volatile(test_ptr, TEST_PATTERN);
    }

    // Read back and verify
    let readback = unsafe { core::ptr::read_volatile(test_ptr) };

    if readback == TEST_PATTERN {
        print_str("NWP DCCM OK\n");
        exit_vcs(0xFF); // TB_CMD_END_SIM_WITH_SUCCESS
        exit_emulator(0x00);
    } else {
        print_str("NWP DCCM FAIL: readback mismatch\n");
        exit_vcs(0x01); // TB_CMD_END_SIM_WITH_FAILURE
        exit_emulator(0x01);
    }
}
