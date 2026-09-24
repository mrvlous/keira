// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Runtime execution, userspace context switching, and terminal loop.

pub mod main_loop;
pub mod userspace;

#[cfg(test)]
mod tests;

pub use main_loop::enter_main_loop;
pub use userspace::enter_userspace;
