// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Crate Root
//!
//! This is the root of the Rust kernel crate. It operates in a fully
//! freestanding environment:
//!   - `no_std`:  No Rust standard library (no OS underneath us!)
//!   - `no_main`: No standard main() entry point (ASM calls us directly)
//!
//! The kernel uses only `core` : the subset of Rust that requires zero
//! OS support. All I/O is performed through FFI calls to C drivers.

#![no_std]
#![no_main]

// Module declarations

/// Kernel entry point (`kernel_main`), called by the ASM trampoline.
pub mod entry;

pub mod fs;
/// I/O subsystem : safe Rust wrappers around C hardware drivers.
pub mod io;
pub mod mem;
pub mod net;
pub mod shell;
pub mod syscall;
pub mod task;

/// Panic handler : required by `no_std` to handle panics gracefully.
pub mod panic;
