// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Inspect and configure Secure Computing (Seccomp) system call filtering sandbox (Syscall 52).

use keira_io::vga;
use keira_task::security::seccomp::{
    allow_syscall, deny_syscall, get_mode, get_stats, reset, set_mode, SeccompMode,
};

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();

    if let Some("-h") | Some("--help") = subcmd {
        print_help();
        return;
    }

    match subcmd {
        None | Some("status") => {
            print_status();
        }
        Some("strict") => {
            set_mode(SeccompMode::Strict);
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[OK] ");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Seccomp Strict Sandbox enabled.\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("     Only read (7, 15), write (1, 8, 16), exit (2), sigreturn (65), and seccomp (52) allowed.\n");
        }
        Some("filter") => {
            set_mode(SeccompMode::Filter);
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[OK] ");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Seccomp Filter Sandbox enabled (Bitmask-based filtering).\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        Some("allow") => {
            if let Some(num_str) = parts.next() {
                if let Ok(num) = num_str.parse::<u64>() {
                    allow_syscall(num);
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[OK] ");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str("Syscall #");
                    vga::print_u64(num);
                    vga::print_str(" added to allowed whitelist mask.\n");
                } else {
                    print_err("Invalid syscall number.");
                }
            } else {
                print_err("Usage: seccomp allow <syscall_number>");
            }
        }
        Some("deny") => {
            if let Some(num_str) = parts.next() {
                if let Ok(num) = num_str.parse::<u64>() {
                    deny_syscall(num);
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[OK] ");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str("Syscall #");
                    vga::print_u64(num);
                    vga::print_str(" removed from allowed whitelist mask.\n");
                } else {
                    print_err("Invalid syscall number.");
                }
            } else {
                print_err("Usage: seccomp deny <syscall_number>");
            }
        }
        Some("reset") | Some("disable") => {
            reset();
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[OK] ");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Seccomp sandbox reset and set to Disabled.\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        Some(other) => {
            print_err("Unknown subcommand. Run 'seccomp --help' for usage.");
        }
    }
}

fn print_status() {
    let mode = get_mode();
    let (checked, violations, last_viol) = get_stats();

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Secure Computing (Seccomp) Sandbox Status:\n");
    vga::print_str("  Operational Mode  : ");

    match mode {
        SeccompMode::Disabled => {
            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("Disabled (Syscalls Unrestricted)\n");
        }
        SeccompMode::Strict => {
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("Strict (Minimal POSIX subset)\n");
        }
        SeccompMode::Filter => {
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("Filter (Dynamic 128-bit Bitmask Whitelist)\n");
        }
    }

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  Syscalls Inspected: ");
    vga::print_u64(checked);
    vga::print_str("\n  Blocked Violations: ");
    if violations > 0 {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
    }
    vga::print_u64(violations);
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);

    if violations > 0 {
        vga::print_str(" (Last Syscall Violation: #");
        vga::print_u64(last_viol);
        vga::print_str(")\n");
    } else {
        vga::print_str("\n");
    }

    vga::print_str("  Enforcement Hook  : Syscall Dispatcher (Syscall 52)\n");
}

fn print_help() {
    vga::print_str("Usage: seccomp [status|strict|filter|allow <num>|deny <num>|reset]\n\n");
    vga::print_str("Description:\n");
    vga::print_str("  Inspect and configure Secure Computing (Seccomp) system call sandboxing (Syscall 52).\n\n");
    vga::print_str("Subcommands:\n");
    vga::print_str(
        "  status            Show current sandbox operational mode and telemetry (default)\n",
    );
    vga::print_str(
        "  strict            Enforce Strict isolation (only read, write, exit, sigreturn)\n",
    );
    vga::print_str("  filter            Enable bitmask-based system call filtering\n");
    vga::print_str("  allow <num>       Permit a specific system call number in filter mode\n");
    vga::print_str("  deny <num>        Block a specific system call number in filter mode\n");
    vga::print_str("  reset             Disable sandbox and clear violation counters\n");
    vga::print_str("  -h, --help        Show this help message and exit\n");
}

fn print_err(msg: &str) {
    vga::set_color(vga::Color::LightRed, vga::Color::Black);
    vga::print_str("[ERROR] ");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str(msg);
    vga::print_str("\n");
}
