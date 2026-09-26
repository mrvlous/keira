// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Autocomplete resolution engine against built-in command tables and VFS paths.

use keira_io::vga;

use crate::autocomplete::word::matcher::find_last_word;
use crate::state::session::{BUFFER_LEN, INPUT_BUFFER};

pub const COMMANDS_LIST: &[&str] = &[
    "bg",
    "bpf",
    "cgroups",
    "copy",
    "cpu",
    "create",
    "delete",
    "devices",
    "disk",
    "download",
    "drivers",
    "drives",
    "edit",
    "epoll",
    "eventfd",
    "ext4",
    "fetch",
    "fg",
    "fileinfo",
    "firewall",
    "folder",
    "framebuffer",
    "futex",
    "go",
    "help",
    "history",
    "hostname",
    "https",
    "initrd",
    "jobs",
    "kcc",
    "kill",
    "kvm",
    "list",
    "lkm",
    "lvm",
    "mac",
    "memory",
    "move",
    "mqueue",
    "network",
    "nvme",
    "perf",
    "power",
    "raid",
    "ramdisk",
    "reset",
    "run",
    "runtime",
    "search",
    "seccomp",
    "smp",
    "stop",
    "swap",
    "sync",
    "syslog",
    "system",
    "tasks",
    "time",
    "timer",
    "tpm",
    "unwind",
    "usb",
    "use",
    "view",
    "watchpoint",
    "wipe",
    "write",
];

pub const STANDARD_PATHS: &[&str] = &[
    "system/",
    "apps/",
    "config/",
    "data/",
    "temp/",
    "system.cfg",
];

/// Resolves tab completion against commands or files and updates the VGA console.
pub unsafe fn handle_autocomplete() {
    let (prefix_start, word) = find_last_word(&INPUT_BUFFER[..BUFFER_LEN]);
    if word.is_empty() {
        return;
    }

    let mut match_count = 0;
    let mut first_match = [0u8; 32];
    let mut first_match_len = 0;

    let is_command = prefix_start == 0;

    if is_command {
        for &cmd in COMMANDS_LIST {
            if cmd.starts_with(word) {
                if match_count == 0 {
                    first_match_len = cmd.len();
                    first_match[..first_match_len].copy_from_slice(cmd.as_bytes());
                }
                match_count += 1;
            }
        }
    } else {
        for &path in STANDARD_PATHS {
            if path.starts_with(word) {
                if match_count == 0 {
                    first_match_len = path.len();
                    first_match[..first_match_len].copy_from_slice(path.as_bytes());
                }
                match_count += 1;
            }
        }

        keira_fs::fat::find_matches(word, |filename| {
            if match_count == 0 {
                first_match_len = filename.len();
                first_match[..first_match_len].copy_from_slice(filename.as_bytes());
            }
            match_count += 1;
        });
    }

    if match_count == 1 {
        let completed = if is_command {
            let mut name = [0u8; 33];
            let len = first_match_len;
            name[..len].copy_from_slice(&first_match[..len]);
            name[len] = b' ';
            (len + 1, name)
        } else {
            let mut name = [0u8; 33];
            let len = first_match_len;
            name[..len].copy_from_slice(&first_match[..len]);
            (len, name)
        };

        let old_word_len = BUFFER_LEN - prefix_start;
        for _ in 0..old_word_len {
            vga::backspace();
        }

        BUFFER_LEN = prefix_start;
        for i in 0..completed.0 {
            INPUT_BUFFER[BUFFER_LEN] = completed.1[i];
            BUFFER_LEN += 1;
        }

        let completion_slice = &INPUT_BUFFER[prefix_start..BUFFER_LEN];
        if let Ok(s) = core::str::from_utf8(completion_slice) {
            vga::print_str(s);
        }
    } else if match_count > 1 {
        vga::print_str("\n");
        let mut printed = 0;
        if is_command {
            for &cmd in COMMANDS_LIST {
                if cmd.starts_with(word) {
                    if printed < 10 {
                        vga::print_str(cmd);
                        vga::print_str("  ");
                        printed += 1;
                    } else {
                        vga::print_str("... ");
                        break;
                    }
                }
            }
        } else {
            for &path in STANDARD_PATHS {
                if path.starts_with(word) && printed < 10 {
                    vga::print_str(path);
                    vga::print_str("  ");
                    printed += 1;
                }
            }
            let mut exceeded = false;
            keira_fs::fat::find_matches(word, |filename| {
                if printed < 10 {
                    vga::print_str(filename);
                    vga::print_str("  ");
                    printed += 1;
                } else {
                    exceeded = true;
                }
            });
            if exceeded {
                vga::print_str("... (and more)");
            }
        }
        vga::print_str("\n");
        crate::print_prompt();
        let buffer_slice = &INPUT_BUFFER[..BUFFER_LEN];
        if let Ok(s) = core::str::from_utf8(buffer_slice) {
            vga::print_str(s);
        }
    }
}
