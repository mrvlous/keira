// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Tar Archive Reader
//!
//! Implements a read-only filesystem reader parsing standard USTAR tar archives.

pub mod reader;

pub use reader::{cat_file, exists, init, list_files, read_file_content};
