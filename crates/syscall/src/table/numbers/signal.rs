// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Signal management and mask system call numbers.

pub const SYS_KILL: u64 = 22;
pub const SYS_SIGACTION: u64 = 64;
pub const SYS_SIGRETURN: u64 = 65;
pub const SYS_SIGPROCMASK: u64 = 81;
pub const SYS_SIGPENDING: u64 = 82;
