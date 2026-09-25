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
        "hostname --help",
        "drives --help",
        "use --help",
        "ramdisk --help",
        "system --help",
        "cpu --help",
        "smp --help",
        "runtime --help",
        "time --help",
        "memory --help",
        "memory -t",
        "memory -p",
        "devices --help",
        "network --help",
        "download --help",
        "initrd --help",
        "wipe --help",
        "reset --help",
        "reboot --help",
        "run --help",
        "tasks --help",
        "tasks -s",
        "stop --help",
        "disk --help",
        "sync --help",
        "list --help",
        "go --help",
        "view --help",
        "write --help",
        "create --help",
        "folder --help",
        "delete --help",
        "edit --help",
        "nano --help",
        "copy --help",
        "help",
        "history --help",
        "move --help",
        "search --help",
        "fileinfo --help",
        "framebuffer --help",
        "usb --help",
        "https --help",
        "drivers --help",
        "lkm --help",
        "lsmod --help",
        "unwind --help",
        "watchpoint --help",
        "power --help",
        "poweroff --help",
        "shutdown --help",
        "perf --help",
        "timer --help",
        "syslog --help",
        "dmesg --help",
        "kvm --help",
        "nvme --help",
        "ext4 --help",
        "cgroups --help",
        "futex --help",
        "bpf --help",
        "tpm --help",
        "swap --help",
        "seccomp --help",
        "epoll --help",
        "eventfd --help",
        "signalfd --help",
        "mac --help",
        "selinux --help",
        "mqueue --help",
        "kill --help",
        "jobs --help",
        "fg --help",
        "bg --help",
        "lvm --help",
        "raid --help",
        "firewall --help",
        "kcc --help",
    ];

    for cmd in help_commands {
        execute_command_inner(cmd);
    }
}

#[test]
fn test_all_shell_commands_default_invocation() {
    let default_commands = [
        "hostname",
        "system",
        "cpu",
        "smp",
        "runtime",
        "time",
        "memory",
        "devices",
        "tasks",
        "disk",
        "drives",
        "list",
        "go",
        "history",
        "framebuffer",
        "usb",
        "drivers",
        "lkm",
        "unwind",
        "watchpoint",
        "perf",
        "timer",
        "syslog",
        "kvm",
        "nvme",
        "ext4",
        "cgroups",
        "futex",
        "bpf",
        "tpm",
        "swap",
        "seccomp",
        "epoll",
        "eventfd",
        "mac",
        "mqueue",
        "jobs",
        "lvm",
        "raid",
        "firewall",
        "network",
        "power",
        "sync",
    ];

    for cmd in default_commands {
        execute_command_inner(cmd);
    }
}
