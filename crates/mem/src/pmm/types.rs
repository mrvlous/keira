// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Physical memory page frame types and fundamental constants.

/// Standard 4KB x86_64 page and physical frame size.
pub const PAGE_SIZE: u64 = 4096;
/// 4KB Page size alias.
pub const PAGE_SIZE_4K: u64 = 4096;
/// 1MB Kernel physical memory base address.
pub const KERNEL_BASE_1MB: u64 = 0x100000;
