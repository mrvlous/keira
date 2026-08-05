#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'bpf_jit'
//!
//! Inspect in-kernel eBPF JIT compiler status (Syscall 59).

use crate::io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: bpf_jit [status]\n\n");
            vga::print_str("Description:\n  Inspect in-kernel eBPF native x86_64 JIT bytecode compiler status (Syscall 59).\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("In-Kernel eBPF JIT Compiler Status (Syscall 59)\n");
        let _ = crate::net::bpf_jit::sys_bpf_jit(core::ptr::null(), 0);
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}
