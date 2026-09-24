// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! File and descriptor I/O system call numbers.

pub const SYS_OPEN: u64 = 6;
pub const SYS_READ: u64 = 7;
pub const SYS_WRITE: u64 = 8;
pub const SYS_CLOSE: u64 = 9;
pub const SYS_LSEEK: u64 = 10;
pub const SYS_SPLICE: u64 = 47;
pub const SYS_VMSPLICE: u64 = 48;
pub const SYS_SYNC: u64 = 70;
pub const SYS_FSYNC: u64 = 71;
pub const SYS_FCNTL: u64 = 72;
pub const SYS_IOCTL: u64 = 73;
pub const SYS_DUP: u64 = 84;
pub const SYS_DUP2: u64 = 85;
