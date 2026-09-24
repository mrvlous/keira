// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! POSIX USTAR header structure and parsing utilities.

pub mod octal;
pub mod types;

pub use self::octal::octal_str_to_u64;
pub use self::types::{UstarHeader, TYPEFLAG_DIRECTORY, TYPEFLAG_REGULAR, TYPEFLAG_REGULAR_ALT};
