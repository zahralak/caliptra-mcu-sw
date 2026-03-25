# Licensed under the Apache-2.0 license
#
# CDC (Clock Domain Crossing) constraints for the Caliptra FPGA design.
#
# This file addresses the following timing issues:
#   - TIMING-6/7/8: clk_pl_0 (core, ~18 MHz) and clk_pl_1 (I3C, 120 MHz)
#     have no common primary clock, node, or expandable period. They are
#     truly asynchronous clocks from independent PS PLLs. The SmartConnect
#     and XPM CDC primitives handle the actual domain crossing safely.
#   - TIMING-9/10: ASYNC_REG properties on CDC synchronizer flip-flops.
#   - AVAL-344: USER_RAM_AVERAGE_ACTIVITY to reduce pessimistic BRAM/URAM
#     clock uncertainty.

# =============================================================================
# 1. Declare clk_pl_0 and clk_pl_1 as asynchronous clock groups
# =============================================================================
# These two clocks originate from separate PS PLL outputs with no phase
# relationship. All crossings between them go through either:
#   - SmartConnect async FIFOs (AXI interconnect m01/m09/m13 ports)
#   - XPM CDC synchronizers in caliptra_ss_top (I3C <-> SS domain)
# This eliminates ~3100 false timing failures (WNS -5.674ns, TNS -16222ns).
set_clock_groups -asynchronous \
    -group [get_clocks clk_pl_0] \
    -group [get_clocks clk_pl_1]

# =============================================================================
# 2. Mark XPM CDC synchronizer registers with ASYNC_REG
# =============================================================================
# The XPM CDC macros should auto-set ASYNC_REG, but TIMING-10 indicates it
# is missing on some instances. This ensures Vivado places the synchronizer
# flip-flops in the same slice and does not optimize them away.
#
# Target the caliptra_ss_top CDC instances and any sync_regs/reset_synchronizer instances.
# Guard each set_property with a length check to avoid warnings on empty cell lists.
foreach pattern {
    {*/xpm_cdc_*/gen_*.u_impl_xilinx/q_o_reg*}
    {*/xpm_cdc_*/*graysync_ff_reg*}
    {*/xpm_cdc_*/*arststages_ff_reg*}
    {*/sync_regs/*/q_o_reg*}
    {*/reset_synchronizer/*/arststages_ff_reg*}
} {
    set cells [get_cells -quiet -hierarchical -filter "NAME =~ $pattern"]
    if {[llength $cells] > 0} {
        set_property ASYNC_REG TRUE $cells
    }
}

# =============================================================================
# 3. Reduce pessimistic RAM switching activity estimate
# =============================================================================
# AVAL-344 warns that USER_RAM_AVERAGE_ACTIVITY is unset, causing worst-case
# BRAM/URAM switching assumptions that inflate clock uncertainty. A value of
# 5 (out of 15) is a reasonable estimate for this design's moderate memory
# access patterns.
set_property USER_RAM_AVERAGE_ACTIVITY 5 [current_design]

# =============================================================================
# 4. Thermal operating conditions
# =============================================================================
# Without these, Vivado assumes worst-case Tj=100C which inflates static power
# estimates. Setting realistic conditions gives accurate power reports.
set_operating_conditions -ambient_temp 25
set_operating_conditions -thetaja 1.0
