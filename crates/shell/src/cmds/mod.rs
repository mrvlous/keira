// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Categorized modular shell command handlers for the Keira Kernel.

pub mod dev;
pub mod fs;
pub mod net;
pub mod proc;
pub mod sec;
pub mod sys;
pub mod util;

// Flat re-exports for backward-compatibility and direct access
pub use dev::*;
pub use fs::*;
pub use net::*;
pub use proc::*;
pub use sec::*;
pub use sys::*;
pub use util::*;
