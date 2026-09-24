// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Standard POSIX error codes and conversion utilities for system call returns.

pub mod codes;
pub mod conv;

pub use codes::*;
pub use conv::{errno_to_ret, ret_to_errno};
