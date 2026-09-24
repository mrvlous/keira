// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Exception vector translation to POSIX signal identifiers and human-readable names.

pub fn exception_vector_to_signal(vector: u64) -> u32 {
    match vector {
        0 => keira_task::signal::SIGFPE,
        4 => keira_task::signal::SIGFPE,
        5 => keira_task::signal::SIGSEGV,
        6 => keira_task::signal::SIGILL,
        7 => keira_task::signal::SIGFPE,
        11 => keira_task::signal::SIGBUS,
        12 => keira_task::signal::SIGBUS,
        13 => keira_task::signal::SIGSEGV,
        14 => keira_task::signal::SIGSEGV,
        16 => keira_task::signal::SIGFPE,
        17 => keira_task::signal::SIGBUS,
        19 => keira_task::signal::SIGFPE,
        _ => keira_task::signal::SIGSEGV,
    }
}

pub fn exception_name(vector: u64) -> &'static str {
    match vector {
        0 => "Division by Zero (#DE)",
        1 => "Debug Exception (#DB)",
        2 => "Non-Maskable Interrupt (NMI)",
        3 => "Breakpoint (#BP)",
        4 => "Overflow (#OF)",
        5 => "Bound Range Exceeded (#BR)",
        6 => "Invalid Opcode (#UD)",
        7 => "Device Not Available (#NM)",
        8 => "Double Fault (#DF)",
        10 => "Invalid TSS (#TS)",
        11 => "Segment Not Present (#NP)",
        12 => "Stack-Segment Fault (#SS)",
        13 => "General Protection Fault (#GP)",
        14 => "Page Fault (#PF)",
        16 => "x87 Floating-Point Exception (#MF)",
        17 => "Alignment Check (#AC)",
        18 => "Machine Check (#MC)",
        19 => "SIMD Floating-Point Exception (#XM)",
        _ => "Unknown Exception",
    }
}

pub fn signal_name(sig: u32) -> &'static str {
    match sig {
        1 => "SIGHUP",
        2 => "SIGINT",
        3 => "SIGQUIT",
        4 => "SIGILL",
        5 => "SIGTRAP",
        6 => "SIGABRT",
        7 => "SIGBUS",
        8 => "SIGFPE",
        9 => "SIGKILL",
        10 => "SIGUSR1",
        11 => "SIGSEGV",
        12 => "SIGUSR2",
        13 => "SIGPIPE",
        14 => "SIGALRM",
        15 => "SIGTERM",
        _ => "UNKNOWN",
    }
}
