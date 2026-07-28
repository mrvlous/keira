// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: ELF Loader Subsystem
//!
//! Provides structures and logic to parse and load ELF64 executable files into memory.

pub mod loader;
pub mod types;

pub use loader::{load_elf, run_user_program, spawn_user_program};
