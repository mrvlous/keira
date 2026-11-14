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

//! POSIX socket abstractions and kernel table management.

pub mod domain;
pub mod handle;
pub mod table;

pub use domain as family;
pub use handle as descriptor;
pub use table as manager;

pub mod sock {
    pub use super::domain::*;
    pub use super::handle::*;
    pub use super::table::*;
}

#[cfg(test)]
mod tests;

pub use domain::*;
pub use handle::*;
pub use table::*;
