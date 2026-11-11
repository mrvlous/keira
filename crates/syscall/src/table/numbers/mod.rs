// SPDX-License-Identifier: GPL-2.0-only

//! System call numbers categorized across kernel subsystems.

pub mod io;
pub mod ipc;
pub mod memory;
pub mod net;
pub mod process;
pub mod signal;
pub mod system;

pub use io::*;
pub use ipc::*;
pub use memory::*;
pub use net::*;
pub use process::*;
pub use signal::*;
pub use system::*;
