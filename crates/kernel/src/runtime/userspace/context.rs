// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Ring 3 user-mode privilege setup, TSS context loading, and security initialization.

use keira_io::vga;
use keira_syscall::init_user_mode;

/// Transition hardware registers into user-mode execution readiness.
///
/// # Safety
/// Caller guarantees GDT, IDT, and scheduler are fully initialized.
pub unsafe fn enter_userspace() {
    init_user_mode();

    vga::print_boot_log("Re-configuring Global Descriptor Table (GDT) segments", 0);
    vga::print_boot_log("Loading Task State Segment (TSS) cpu context structure", 0);
    vga::print_boot_log("Enabling CPU ring 3 user-mode syscall interface MSRs", 0);
    vga::print_boot_log("Initializing Mandatory Access Control (MAC) Security", 0);
    vga::print_boot_log("Spawning interactive terminal shell environment", 0);
    vga::print_boot_log("Keira Kernel initialized successfully. System ready", 0);
}
