// Licensed under the Apache-2.0 license

//! Test modules for Network Coprocessor ROM
//!
//! Each test is gated by a cargo feature (e.g., `test-hello-world`).
//! When no test feature is enabled, the default firmware behavior runs.

#[cfg(feature = "test-hello-world")]
pub mod hello_world;

#[cfg(feature = "test-dccm")]
pub mod dccm;

#[cfg(feature = "test-exception")]
pub mod exception;
