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
