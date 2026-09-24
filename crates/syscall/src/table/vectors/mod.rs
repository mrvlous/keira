// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! System call name lookup and vector categorization.

pub mod categorize;
pub mod names;

pub use categorize::{
    is_io_syscall, is_ipc_syscall, is_memory_syscall, is_process_syscall, is_valid_syscall,
};
pub use names::syscall_name;
