// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! FAT on-disk structures, directory entries, VFAT LFN descriptors, and volume layout.

pub mod entry;
pub mod lfn;
pub mod volume;

pub use self::entry::DirectoryEntry;
pub use self::lfn::{LfnAccumulator, LfnEntry};
pub use self::volume::{Fat16Volume, FoundEntry};
