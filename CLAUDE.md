# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

**caliptra-mcu-sw** is the firmware and software for the Caliptra MCU, a RISC-V security controller in the Caliptra Subsystem. It is a Rust workspace targeting `riscv32imc-unknown-none-elf` with Tock OS as the runtime kernel.

Two-repo structure:
- This repo (`caliptra-mcu-sw`): Rust firmware workspace
- `hw/caliptra-ss` submodule: Caliptra Subsystem RTL (SystemVerilog hardware design)

Always ensure submodules are initialized: `git submodule update --init --recursive`

## Build Commands

Everything runs through `cargo xtask` (no Makefiles or bash scripts for SW builds):

```bash
cargo xtask precheckin          # Full pre-push check (fmt, clippy, headers, deps, builds, tests)
cargo xtask clippy              # Clippy with -D warnings
cargo xtask format              # cargo fmt --check --all
cargo xtask header-check        # Apache-2.0 license header in all source files
cargo xtask header-fix          # Auto-add missing headers
cargo xtask deps                # Verify all deps use dep.workspace = true
cargo xtask cargo-lock          # Verify Cargo.lock is committed
cargo xtask registers-autogen   # Regenerate register files from RDL (after hw/caliptra-ss update)
cargo xtask rom-build           # Build MCU ROM only
cargo xtask runtime-build       # Build MCU runtime only
cargo xtask all-build --platform emulator  # Build all firmware into target/all-fw.zip
cargo xtask runtime             # Build + run both emulators (MCU + Caliptra Core)
```

## Testing

Tests use `cargo nextest` (not `cargo test`):

```bash
cargo xtask test                                    # Run all tests (nightly-emulator profile)
cargo nextest run -p <crate> <test_name>            # Run a single test
cargo nextest run -p emulator-tests <test_name>     # Run a specific integration test
```

Integration tests run single-threaded (emulators bind to ports). Tests use cargo features as test selectors — building runtime with a `test-*` feature produces a binary that runs that test scenario.

Environment variables for pre-built artifacts (avoids recompilation):
- `CPTRA_FIRMWARE_BUNDLE` — path to `all-fw.zip`
- `CPTRA_EMULATOR_BUNDLE` — path to `emulators.zip`

## RTL Simulation (VCS)

For hardware simulation of the Caliptra Subsystem (in `hw/caliptra-ss/`):

```bash
export CALIPTRA_SS_ROOT=/path/to/hw/caliptra-ss
export CALIPTRA_ROOT=${CALIPTRA_SS_ROOT}/third_party/caliptra-rtl

mkdir sim_run && cd sim_run
make -f ${CALIPTRA_SS_ROOT}/tools/scripts/Makefile \
  TESTNAME=mcu_hello_world \
  CALIPTRA_TESTNAME=cptra_hello_world \
  vcs
```

Requires: VCS, RISC-V toolchain (`riscv64-unknown-elf-gcc`), Avery AXI VIP (`AVERY_HOME`, `AVERY_PLI`).

The `vcs` target builds hex files (mcu_program.hex, nwp_program.hex, cptra_program.hex), compiles RTL, and runs simulation.

## Architecture

### Firmware Layers

- **MCU ROM** (`rom/`, `platforms/emulator/rom/`): Cold boot, image verification, firmware loading. Must be panic-free (enforced by `precheckin`).
- **MCU Runtime** (`runtime/`, `platforms/emulator/runtime/`): Tock OS kernel with capsules for each peripheral. Test scenarios selected via `test-*` cargo features.
- **Network ROM** (`network/rom/`): Firmware for the Network Coprocessor (NWP), a second RISC-V core.
- **Caliptra Core FW**: Built via `caliptra-builder` from `caliptra-sw` (pinned git rev).

### Emulator

The MCU emulator (`emulator/app/`) is a host-side binary embedding:
- RISC-V CPU emulator for MCU
- Separate Caliptra Core emulator thread
- Optional Network Coprocessor CPU
- Simulated peripherals (I3C, flash, DMA, mailbox, OTP, lifecycle controller)

### Register Auto-Generation

Rust register definitions are generated from SystemRDL files in `hw/caliptra-ss/`. After updating the `hw/caliptra-ss` submodule, run `cargo xtask registers-autogen` and commit the results.

### Hardware Revisions

- **2.0** (default): standard Caliptra Subsystem
- **2.1** (`hw-2-1` feature): adds ML-KEM, I3C AXI recovery bypass. Integration tests use 2.1.

## Key Constraints

- **All workspace deps must use `dep.workspace = true`** — no inline version strings in crate `Cargo.toml` files (enforced by `cargo xtask deps`).
- **Cargo.lock must be committed** — CI runs `cargo tree --locked`.
- **Apache-2.0 header required** in first 3 lines of all source files (.rs, .sv, .c, .h, .toml, .py, etc.).
- **ROM must be panic-free** — `precheckin` checks for `panic_is_possible` symbol in ROM ELF.
- **RISC-V build flags** (set in `.cargo/config.toml`): `panic=abort`, extensions `+zba,+zbb,+zbc,+zbs`, `relocation-model=static`, `linker=rust-lld`, `-nmagic`, `-icf=all`.
- **Excluded from vendoring rules**: `libtock/` directory is vendored third-party code.

## NWP (Network Processor) Integration

The NWP is a second VeeR EL2 RISC-V core with namespace prefix `css_nwp0_` (vs `css_mcu0_` for MCU).

NWP memory map:
| Region | Address | Size |
|--------|---------|------|
| DCCM | 0x30000000 | 64 KB |
| PIC | 0xB0000000 | 32 KB |
| ROM (reset vector) | 0x90000000 | 256 KB |
| ICCM | 0xC0000000 | disabled |

NWP firmware: `network/config/` (memory map), `network/rom/` (firmware crate). The `DEFAULT_NETWORK_MEMORY_MAP` const in `network/config/src/lib.rs` is used by `build.rs` directly (not the `Default` impl).

## RTL Conventions (hw/caliptra-ss/)

- Verible linter rules enforced: `explicit-parameter-storage-type`, `generate-label` (prefix `g_` or `gen_`), `line-length` (max 100), `no-tabs`, `forbid-defparam`
- VeeR EL2 core files under `src/riscv_core/veer_el2_nwp/` are auto-generated by the VeeR config tool — lint issues there are upstream
- Compilation order defined by `config/compilespecs.yml` and per-component `config/compile.yml` files
- Verilog file lists: `src/integration/config/caliptra_ss_top.vf` and `caliptra_ss_top_tb.vf`
