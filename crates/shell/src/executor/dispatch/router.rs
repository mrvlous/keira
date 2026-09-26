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

    // PATH resolution: strip /system/bin/ prefix if present
    let command = raw_command
        .strip_prefix("/system/bin/")
        .unwrap_or(raw_command);

    match command {
        "hostname" => crate::cmds::hostname::run(&mut parts),
        "drives" => crate::cmds::drives::run(&mut parts),
        "use" => crate::cmds::r#use::run(&mut parts),
        "ramdisk" => crate::cmds::ramdisk::run(&mut parts),
        "system" => crate::cmds::sys::system::run(&mut parts),
        "cpu" => crate::cmds::cpu::run(&mut parts),
        "smp" => crate::cmds::smp::run(&mut parts),
        "runtime" => crate::cmds::runtime::run(&mut parts),
        "time" => crate::cmds::time::run(&mut parts),
        "memory" => crate::cmds::memory::run(&mut parts),
        "devices" => crate::cmds::devices::run(&mut parts),
        "network" => crate::cmds::network::run(&mut parts),
        "download" => crate::cmds::download::run(&mut parts),
        "fetch" => crate::cmds::fetch::run(&mut parts),
        "initrd" => crate::cmds::initrd::run(&mut parts),
        "wipe" => crate::cmds::wipe::run(&mut parts),
        "reset" | "reboot" => crate::cmds::reset::run(&mut parts),
        "run" => crate::cmds::run::run(&mut parts),
        "tasks" => crate::cmds::tasks::run(&mut parts),
        "stop" => crate::cmds::stop::run(&mut parts),
        "disk" => crate::cmds::disk::run(&mut parts),
        "sync" => crate::cmds::sync::run(&mut parts),

        "list" => crate::cmds::list::run(&mut parts),
        "go" => crate::cmds::go::run(&mut parts),
        "view" => crate::cmds::view::run(&mut parts),
        "write" => crate::cmds::write::run(&mut parts),
        "create" => crate::cmds::create::run(&mut parts),
        "folder" => crate::cmds::folder::run(&mut parts),
        "delete" => crate::cmds::delete::run(&mut parts),
        "edit" | "nano" => crate::cmds::edit::run(&mut parts),
        "copy" => crate::cmds::copy::run(&mut parts),
        "help" => crate::cmds::help::run(&mut parts),
        "history" => crate::cmds::history::run(&mut parts),
        "move" => crate::cmds::r#move::run(&mut parts),
        "search" => crate::cmds::search::run(&mut parts),
        "fileinfo" => crate::cmds::fileinfo::run(&mut parts),
        "framebuffer" => crate::cmds::framebuffer::run(&mut parts),
        "usb" => crate::cmds::usb::run(&mut parts),
        "https" => crate::cmds::https::run(&mut parts),

        "drivers" => crate::cmds::drivers::run(&mut parts),
        "lkm" | "lsmod" => crate::cmds::lkm::run(&mut parts),
        "unwind" => crate::cmds::unwind::run(&mut parts),
        "watchpoint" => crate::cmds::watchpoint::run(&mut parts),
        "power" | "poweroff" | "shutdown" => crate::cmds::power::run(&mut parts),
        "perf" => crate::cmds::perf::run(&mut parts),
        "timer" => crate::cmds::timer::run(&mut parts),
        "syslog" | "dmesg" => crate::cmds::syslog::run(&mut parts),
        "kvm" => crate::cmds::kvm::run(&mut parts),
        "nvme" => crate::cmds::nvme::run(&mut parts),
        "ext4" => crate::cmds::ext4::run(&mut parts),
        "cgroups" => crate::cmds::cgroups::run(&mut parts),
        "futex" => crate::cmds::futex::run(&mut parts),
        "bpf" => crate::cmds::bpf::run(&mut parts),
        "tpm" => crate::cmds::tpm::run(&mut parts),
        "swap" => crate::cmds::swap::run(&mut driver_swap(&mut parts)),
        "seccomp" => crate::cmds::seccomp::run(&mut parts),
        "epoll" => crate::cmds::epoll::run(&mut parts),
        "eventfd" | "signalfd" => crate::cmds::eventfd::run(&mut parts),
        "mac" | "selinux" => crate::cmds::mac::run(&mut parts),
        "mqueue" => crate::cmds::mqueue::run(&mut parts),
        "kill" => crate::cmds::kill::run(&mut parts),
        "jobs" => crate::cmds::jobs::run(&mut parts),
        "fg" => crate::cmds::fg::run(&mut parts),
        "bg" => crate::cmds::bg::run(&mut parts),
        "lvm" => crate::cmds::lvm::run(&mut parts),
        "raid" => crate::cmds::raid::run(&mut parts),
        "firewall" => crate::cmds::firewall::run(&mut parts),
        "kcc" => crate::cmds::proc::kcc::run(&mut parts),
        _ => {
            if crate::cmds::run::run_direct(command) {
                return;
            }

            let found_in_path = unsafe {
                let mut path_buf = [0u8; 64];
                let cmd_bytes = command.as_bytes();

                let prefix_sys = b"/system/bin/";
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
                vga::print_str("Binary found in /system/bin. Use 'run /system/bin/");
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

#[inline]
fn driver_swap<'a>(parts: &mut core::str::SplitWhitespace<'a>) -> core::str::SplitWhitespace<'a> {
    parts.clone()
}
