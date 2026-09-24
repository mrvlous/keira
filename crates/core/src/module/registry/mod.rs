// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Global kernel module registration table and lookup registry subsystem.

pub mod manager;
pub mod table;

pub use manager::{get_module, get_modules_snapshot, register_module, unregister_module};
pub use table::{MAX_MODULES, MODULE_TABLE};
