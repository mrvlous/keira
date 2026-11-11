// SPDX-License-Identifier: GPL-2.0-only

//! Modular domain-specific system call handler functions.

pub mod fs;
pub mod ipc;
pub mod memory;
pub mod net;
pub mod poll;
pub mod process;
pub mod resource;
pub mod select;
pub mod signal;
pub mod system;

pub use fs::*;
pub use ipc::*;
pub use memory::*;
pub use net::*;
pub use poll::*;
pub use process::*;
pub use resource::*;
pub use select::*;
pub use signal::*;
pub use system::*;
