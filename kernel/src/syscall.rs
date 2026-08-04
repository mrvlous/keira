// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: System Call Subsystem
//!
//! Provides the primary interfaces for system calls, CPU exceptions, and TSS management.

pub mod exception;
pub mod futex;
pub mod handler;
pub mod tss;

pub use tss::{init_user_mode, TaskStateSegment, TSS};
