// SPDX-License-Identifier: GPL-2.0-only

//! Global kernel module registration table and lookup registry subsystem.

pub mod manager;
pub mod table;

pub use manager::{get_module, get_modules_snapshot, register_module, unregister_module};
pub use table::{MAX_MODULES, MODULE_TABLE};
