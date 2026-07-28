// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Filesystem Module Root
//!
//! Provides the primary file system interfaces for Keira Kernel, including
//! ELF loader, FAT16 filesystem support, and TAR RAM disk parsing.

pub mod elf;
pub mod fat;
pub mod lock;
pub mod tar;
pub mod vfs;
