// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Built-in command dispatcher and binary executable launcher.

use keira_io::vga;

/// Dispatches a command line string to built-in commands or executes binary from filesystem.
pub fn execute_command_inner(cmd: &str) {
    let mut parts = cmd.split_whitespace();
    let raw_command = match parts.next() {
        Some(c) => c,
        None => return,
    };

    // PATH resolution: strip /bin/ prefix if present
    let command = raw_command.strip_prefix("/bin/").unwrap_or(raw_command);

    match command {
        "system" => crate::cmds::sys::system::run(&mut parts),
        "cpu" => crate::cmds::cpu::run(&mut parts),
        "smp" => crate::cmds::smp::run(&mut parts),
        "memory" => crate::cmds::memory::run(&mut parts),
        "devices" => crate::cmds::devices::run(&mut parts),
        "drivers" => crate::cmds::drivers::run(&mut parts),
        "disk" => crate::cmds::disk::run(&mut parts),
        "sync" => crate::cmds::sync::run(&mut parts),
        "syslog" => crate::cmds::syslog::run(&mut parts),
        "unwind" => crate::cmds::unwind::run(&mut parts),
        "watchpoint" => crate::cmds::watchpoint::run(&mut parts),
        "run" => crate::cmds::run::run(&mut parts),
        "reset" | "reboot" => crate::cmds::reset::run(&mut parts),
        "power" | "poweroff" | "shutdown" => crate::cmds::power::run(&mut parts),
        "help" => crate::cmds::help::run(&mut parts),
        _ => {
            if crate::cmds::run::run_direct_with_parts(command, &mut parts) {
                return;
            }

            let found_in_path = unsafe {
                let mut path_buf = [0u8; 64];
                let cmd_bytes = command.as_bytes();

                let prefix_sys = b"/bin/";
                let mut path_sys_ok = false;
                if prefix_sys.len() + cmd_bytes.len() < 64 {
                    path_buf[..prefix_sys.len()].copy_from_slice(prefix_sys);
                    path_buf[prefix_sys.len()..prefix_sys.len() + cmd_bytes.len()]
                        .copy_from_slice(cmd_bytes);
                    let path_str =
                        core::str::from_utf8(&path_buf[..prefix_sys.len() + cmd_bytes.len()])
                            .unwrap_or("");
                    let in_fat = if let Ok((dir_cluster, filename)) =
                        keira_fs::fat::resolve_path(path_str)
                    {
                        keira_fs::fat::find_entry(filename, dir_cluster).is_ok()
                    } else {
                        false
                    };
                    path_sys_ok = in_fat || keira_fs::tar::exists(path_str);
                }

                path_sys_ok
            };

            if found_in_path {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("Binary found in /bin. Use 'run /bin/");
                vga::print_str(command);
                vga::print_str("' to execute.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            } else {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Unknown command: ");
                vga::print_str(command);
                vga::print_str(". Type 'help' for available commands.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}
