// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Modular domain-specific system call handler functions.

pub mod fs;
pub mod ipc;
pub mod memory;
pub mod net;
pub mod process;
pub mod signal;
pub mod system;

pub use fs::*;
pub use ipc::*;
pub use memory::*;
pub use net::*;
pub use process::*;
pub use signal::*;
pub use system::*;
