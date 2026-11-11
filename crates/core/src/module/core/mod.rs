// SPDX-License-Identifier: GPL-2.0-only

//! Core kernel module descriptors and state definitions.

pub mod descriptor;
pub mod state;

pub use descriptor::{KernelModule, MAX_MODULE_NAME};
pub use state::ModuleState;
