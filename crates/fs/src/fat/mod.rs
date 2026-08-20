// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! FAT16 filesystem implementation with VFAT long file names and cluster management.

pub mod cluster;
pub mod dir;
pub mod file;
pub mod path;
pub mod table;
pub mod types;
pub mod volume;

pub use cluster::*;
pub use dir::*;
pub use file::*;
pub use path::*;
pub use table::*;
pub use types::*;
pub use volume::*;
