// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'help' shell command.

use keira_io::vga;

pub fn run(_parts: &mut core::str::SplitWhitespace) {
    let bg = vga::Color::Black;
    let fg = vga::Color::White;

    vga::set_color(fg, bg);
    vga::print_str("Keira Kernel Diagnostic & Emergency Debugger (Ring 0 v0.7.0)\n");
    vga::print_str("Built-in Commands:\n");
    vga::print_str("  system         Display system summary, OS version and uptime\n");
    vga::print_str("  cpu            Display CPU model, features and cache topology\n");
    vga::print_str("  smp            Display symmetric multiprocessing core status\n");
    vga::print_str("  memory         Display physical and virtual memory usage (mem)\n");
    vga::print_str(
        "  slabinfo       Display slab memory allocator caches and hit metrics (slab)\n",
    );
    vga::print_str("  devices        Enumerate PCI buses and peripheral hardware (pci)\n");
    vga::print_str("  drivers        List registered block, char and network drivers\n");
    vga::print_str("  regs           Dump CPU general-purpose and control registers\n");
    vga::print_str("  irq            Display interrupt request lines and handler counters\n");
    vga::print_str("Emergency Control:\n");
    vga::print_str("  disk           Inspect storage drives and block device geometry\n");
    vga::print_str("  dcache         Inspect VFS in-memory dentry cache telemetry and hits\n");
    vga::print_str("  mount [vol]    Display mounted filesystems or mount storage volume\n");
    vga::print_str("  pipes          Inspect active IPC pipe streams and buffer allocations\n");
    vga::print_str("  sync           Flush dirty block cache buffers to non-volatile disk\n");
    vga::print_str("  syslog         Display circular kernel log buffer messages (dmesg)\n");
    vga::print_str("  unwind         Print kernel execution stack trace backtrace\n");
    vga::print_str("  watchpoint     Configure memory access watchpoints and breakpoints\n");
    vga::print_str("  wipe           Securely zero volatile memory regions and caches\n");
    vga::print_str("Execution & Power:\n");
    vga::print_str("  sh             Launch canonical POSIX userspace shell in Ring 3\n");
    vga::print_str("  run <path>     Execute freestanding ELF binary from filesystem\n");
    vga::print_str("  reset          Reboot system via ACPI or 8042 keyboard controller\n");
    vga::print_str("  power          Power off system via ACPI shutdown\n");
    vga::print_str("  help           Display this reference manual\n");
    vga::print_str("Userspace Integration:\n");
    vga::print_str("  Core Utilities (/bin): 41 native ELF binaries execute via sh or run\n");
}
