// Licensed under the Apache-2.0 license

//! Integration test for NWP exception handling — triggers illegal instruction,
//! verifies exception handler runs and resumes execution.

#[cfg(test)]
mod test {
    use crate::test::{start_runtime_hw_model, TestParams, TEST_LOCK};
    use mcu_hw_model::McuHwModel;

    #[test]
    #[cfg_attr(feature = "fpga_realtime", ignore)]
    fn test_nwp_exception() {
        let lock = TEST_LOCK.lock().unwrap();
        lock.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let mut hw = start_runtime_hw_model(TestParams {
            include_network_rom: true,
            network_feature: Some("test-exception"),
            rom_only: true,
            ..Default::default()
        });

        assert!(
            hw.has_network_cpu(),
            "Network CPU should be initialized"
        );

        const MAX_CYCLES: u64 = 200_000;
        hw.step_until(|m| {
            if m.cycle_count() >= MAX_CYCLES {
                return true;
            }
            if let Some(output) = m.network_uart_output() {
                if output.contains("NWP exception test PASS")
                    || output.contains("NWP exception test FAIL")
                {
                    return true;
                }
            }
            false
        });

        let output = hw
            .network_uart_output()
            .expect("Network CPU should have UART output");
        println!("NWP UART output:\n{}", output);

        assert!(
            output.contains("NWP exception test PASS"),
            "NWP exception handler test should pass"
        );

        lock.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}
