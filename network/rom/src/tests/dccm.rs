// Licensed under the Apache-2.0 license

//! NWP DCCM test — validates data integrity across full DCCM range.
//!
//! Writes walking-1 patterns, then 0x00000000 and 0xFFFFFFFF, reads back and verifies.
//! Only tests the region between BSS_END and the stack (avoids .data/.bss and stack).

use network_config::DEFAULT_NETWORK_MEMORY_MAP;

extern "C" {
    static BSS_END: u32;
}

/// Run the DCCM data integrity test.
pub fn run(
    print_str: fn(&str),
    print_hex: fn(u32),
    exit_emulator: fn(u32) -> !,
    exit_vcs: fn(u8),
) -> ! {
    print_str("NWP test: dccm\n");

    let dccm_end = DEFAULT_NETWORK_MEMORY_MAP.dccm_offset
        + DEFAULT_NETWORK_MEMORY_MAP.dccm_size;

    // Safe test region: from BSS_END (aligned) to stack bottom
    // Stack is at top of DCCM (8KB), so stack bottom = dccm_end - stack_size
    let stack_bottom = dccm_end - DEFAULT_NETWORK_MEMORY_MAP.rom_stack_size;

    let test_start = core::ptr::addr_of!(BSS_END) as u32;
    let test_start = (test_start + 3) & !3; // Align up to 4

    if test_start >= stack_bottom {
        print_str("NWP DCCM FAIL: no free region between BSS and stack\n");
        exit_vcs(0x01);
        exit_emulator(0x01);
    }

    let num_words = (stack_bottom - test_start) / 4;
    let mut errors: u32 = 0;

    // Test 1: Walking-1 pattern
    print_str("  Walking-1 pattern...\n");
    for i in 0..num_words {
        let pattern = 1u32 << (i % 32);
        let addr = (test_start + i * 4) as *mut u32;
        unsafe {
            core::ptr::write_volatile(addr, pattern);
        }
    }
    for i in 0..num_words {
        let expected = 1u32 << (i % 32);
        let addr = (test_start + i * 4) as *const u32;
        let actual = unsafe { core::ptr::read_volatile(addr) };
        if actual != expected {
            errors += 1;
        }
    }

    // Test 2: All zeros
    print_str("  All-zeros pattern...\n");
    for i in 0..num_words {
        let addr = (test_start + i * 4) as *mut u32;
        unsafe {
            core::ptr::write_volatile(addr, 0x0000_0000);
        }
    }
    for i in 0..num_words {
        let addr = (test_start + i * 4) as *const u32;
        let actual = unsafe { core::ptr::read_volatile(addr) };
        if actual != 0 {
            errors += 1;
        }
    }

    // Test 3: All ones
    print_str("  All-ones pattern...\n");
    for i in 0..num_words {
        let addr = (test_start + i * 4) as *mut u32;
        unsafe {
            core::ptr::write_volatile(addr, 0xFFFF_FFFF);
        }
    }
    for i in 0..num_words {
        let addr = (test_start + i * 4) as *const u32;
        let actual = unsafe { core::ptr::read_volatile(addr) };
        if actual != 0xFFFF_FFFF {
            errors += 1;
        }
    }

    if errors == 0 {
        print_str("NWP DCCM test PASS (");
        print_hex(num_words);
        print_str(" words tested)\n");
        exit_vcs(0xFF);
        exit_emulator(0x00);
    } else {
        print_str("NWP DCCM test FAIL (");
        print_hex(errors);
        print_str(" errors)\n");
        exit_vcs(0x01);
        exit_emulator(0x01);
    }
}
