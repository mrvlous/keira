// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Trigger kernel stack frame unwinder backtrace (Syscall 37).

#![allow(unused_variables, unused_unsafe)]

use keira_io::vga;

#[inline(never)]
pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: unwind\n\n");
            vga::print_str(
                "Description:\n  Trigger kernel stack frame unwinder backtrace (Syscall 37).\n",
            );
        }
        return;
    }

    #[cfg(target_os = "none")]
    keira_arch::unwind::unwind_stack();
}
