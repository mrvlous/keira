// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Primitives for copying data between kernel address space and user address space.

pub mod buffer;
pub mod typed;

pub use buffer::{copy_from_user, copy_to_user};
pub use typed::{copy_val_from_user, copy_val_to_user};
