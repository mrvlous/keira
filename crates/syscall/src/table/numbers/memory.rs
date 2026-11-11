// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Virtual memory and heap system call numbers.

pub const SYS_SBRK: u64 = 11;
pub const SYS_BRK: u64 = 12;
pub const SYS_MMAP: u64 = 20;
pub const SYS_MUNMAP: u64 = 21;
pub const SYS_MPROTECT: u64 = 31;
pub const SYS_MADVISE: u64 = 32;
pub const SYS_MSYNC: u64 = 83;
