#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'unwind'
//!
//! Trigger kernel stack frame unwinder backtrace (Syscall 37).

use crate::io::vga;

#[inline(never)]
pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: unwind [backtrace]\n\n");
            vga::print_str("Description:\n  Trigger kernel stack frame unwinder backtrace & ptrace tracing (Syscall 37).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    crate::arch::unwind::unwind_stack();
}
