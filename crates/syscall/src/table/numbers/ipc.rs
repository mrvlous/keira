// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Inter-process communication and synchronization system call numbers.

pub const SYS_PIPE: u64 = 23;
pub const SYS_SHMGET: u64 = 28;
pub const SYS_SHMAT: u64 = 29;
pub const SYS_IO_URING_SETUP: u64 = 38;
pub const SYS_IO_URING_ENTER: u64 = 39;
pub const SYS_FUTEX: u64 = 40;
pub const SYS_EVENTFD: u64 = 50;
pub const SYS_SIGNALFD: u64 = 51;
pub const SYS_EPOLL_CREATE: u64 = 55;
pub const SYS_EPOLL_CTL: u64 = 56;
pub const SYS_EPOLL_WAIT: u64 = 57;
pub const SYS_MQ_OPEN: u64 = 58;
pub const SYS_SHM_SEM: u64 = 75;
