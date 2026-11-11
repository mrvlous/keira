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

//! Categorized modular shell command handlers for the Keira Kernel.

pub mod dev;
pub mod fs;
pub mod proc;
pub mod sys;
pub mod util;

#[cfg(test)]
mod tests;

pub use dev::*;
pub use fs::*;
pub use proc::*;
pub use sys::*;
pub use util::*;
