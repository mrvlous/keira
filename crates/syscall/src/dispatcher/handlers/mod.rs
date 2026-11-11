// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

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
