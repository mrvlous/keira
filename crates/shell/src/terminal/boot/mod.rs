// SPDX-License-Identifier: GPL-2.0-only

//! Startup boot sequencing and event loop execution.

pub mod pending;
pub mod script;

pub use pending::process_pending;
pub use script::run_boot_script;
