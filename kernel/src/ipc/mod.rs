// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Inter-Process Communication Module Root

pub mod pipe;
pub mod shm;

/// Asynchronous Kernel I/O Engine (io_uring)
pub mod io_uring;

/// Zero-Copy Kernel Pipe Splice Subsystem
pub mod splice;

/// EventFD & SignalFD Subsystem
pub mod eventfd;

/// Epoll Scalable I/O Event Notification Engine
pub mod epoll;

/// POSIX Message Queue IPC Subsystem
pub mod mqueue;

/// io_uring Async I/O Worker Thread Pool Engine
pub mod io_worker;
