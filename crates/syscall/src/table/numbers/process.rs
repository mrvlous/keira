// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Process lifecycle and identity system call numbers.

pub const SYS_PUTC: u64 = 1;
pub const SYS_EXIT: u64 = 2;
pub const SYS_SLEEP: u64 = 3;
pub const SYS_UPTIME: u64 = 4;
pub const SYS_EXEC: u64 = 5;
pub const SYS_WAIT: u64 = 13;
pub const SYS_GETPID: u64 = 14;
pub const SYS_GETCWD: u64 = 15;
pub const SYS_CHDIR: u64 = 16;
pub const SYS_FORK: u64 = 30;
pub const SYS_CLONE_THREAD: u64 = 41;
pub const SYS_PRCTL: u64 = 59;
pub const SYS_GETUID: u64 = 60;
pub const SYS_SETUID: u64 = 61;
pub const SYS_WAITPID: u64 = 62;
pub const SYS_GETPPID: u64 = 63;
pub const SYS_GETGID: u64 = 68;
pub const SYS_SETGID: u64 = 69;
