// SPDX-License-Identifier: GPL-2.0-only

//! System call interfaces for runtime module lifecycle management.

pub mod handlers;

pub use handlers::{sys_delete_module, sys_init_module};
