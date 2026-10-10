// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Integration tests across all categorized shell commands.

use super::*;
use crate::execute_command_inner;

#[test]
fn test_all_command_categories_reexports() {
    let mut args = "help".split_whitespace();
    args.next();
    help::run(&mut args);

    let mut args = "system --help".split_whitespace();
    args.next();
    system::run(&mut args);
}

#[test]
fn test_all_shell_commands_dispatch() {
    let help_commands = [
        "system --help",
        "cpu --help",
        "smp --help",
        "memory --help",
        "memory -t",
        "memory -p",
        "devices --help",
        "drivers --help",
        "disk --help",
        "sync --help",
        "syslog --help",
        "unwind --help",
        "watchpoint --help",
        "run --help",
        "reset --help",
        "reboot --help",
        "power --help",
        "poweroff --help",
        "shutdown --help",
        "help",
    ];

    for cmd in help_commands {
        execute_command_inner(cmd);
    }
}

#[test]
fn test_all_shell_commands_default_invocation() {
    let default_commands = [
        "system",
        "cpu",
        "smp",
        "memory",
        "devices",
        "drivers",
        "disk",
        "sync",
        "syslog",
        "unwind",
        "watchpoint",
        "run",
        "power",
        "help",
    ];

    for cmd in default_commands {
        execute_command_inner(cmd);
    }
}
