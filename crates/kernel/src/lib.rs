// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![no_std]
#![no_main]

//! Master kernel assembly crate linking all modular subsystems into a freestanding binary.

pub mod entry;
pub mod panic;

pub use keira_arch as arch;
pub use keira_core as core_subsystem;
pub use keira_crypto as crypto;
pub use keira_fs as fs;
pub use keira_io as io;
pub use keira_ipc as ipc;
pub use keira_mem as mem;
pub use keira_net as net;
pub use keira_shell as shell;
pub use keira_syscall as syscall;
pub use keira_task as task;

pub use entry::kernel_main;
