// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! User mode initialization, descriptor configuration, and MSR setup.

pub mod gdt;
pub mod init;
pub mod msr;

pub use gdt::configure_gdt_tss;
pub use init::init_user_mode;
pub use msr::configure_syscall_msrs;
