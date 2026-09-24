// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Path resolution, 8.3 short filename formatting, and VFAT long filename accumulator.

pub mod lfn;
pub mod resolver;
pub mod sfn;

pub use self::lfn::{accumulate_lfn, get_lfn_utf8};
pub use self::resolver::{find_entry, resolve_path, sanitize_path};
pub use self::sfn::{filename_to_8_3, format_filename};
