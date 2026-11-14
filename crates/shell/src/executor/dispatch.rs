// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Subsystem module: dispatch.

pub mod entry {
use keira_io::vga;

use crate::executor::env::expand::expand_env_vars;
use crate::executor::pipeline::pipe::execute_pipeline;
use crate::executor::pipeline::redirect::{
    execute_with_redirection, parse_input_redirection, parse_output_redirection,
};

/// Primary entry point: parses, expands environment variables and executes a shell command line.
pub fn execute_command(cmd: &str) {
    let mut exp_buf = [0u8; 512];
    let exp_len = expand_env_vars(cmd, &mut exp_buf);
    let expanded_cmd = match core::str::from_utf8(&exp_buf[..exp_len]) {
        Ok(s) => s,
        Err(_) => cmd,
    };

    let trimmed = expanded_cmd.trim();

    // 1. Pipeline check (has |)
    if execute_pipeline(trimmed) {
        return;
    }

    // 2. Input Redirection check (has <)
    let (actual_cmd, input_file) = parse_input_redirection(trimmed);

    if let Some(filename) = input_file {
        unsafe {
            let mut file_buf = [0u8; 4096];
            let bytes_read = match keira_fs::vfs::read_file(filename, &mut file_buf) {
                Ok(len) => len,
                Err(e) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error reading input redirection file: ");
                    vga::print_str(e);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    return;
                }
            };

            keira_io::vga::PIPE_BUFFER = [0; 4096];
            keira_io::vga::PIPE_BUFFER[..bytes_read].copy_from_slice(&file_buf[..bytes_read]);
            keira_io::vga::PIPE_LEN = bytes_read;
            keira_io::vga::PIPE_ACTIVE = true;
            keira_io::vga::PIPE_READ_INDEX = 0;
        }
    }

    // 3. Output Redirection check (has > or >>)
    let (final_cmd, redirection_target, is_append) = parse_output_redirection(actual_cmd);

    unsafe {
        execute_with_redirection(final_cmd, redirection_target, is_append, input_file);
    }
}

}

pub use self::entry::*;
pub mod hardware {
use keira_io::vga;

/// Re-export of VGA initialization for command routines.
pub fn vga_init() {
    vga::init();
}

/// CMOS Real-Time Clock binary timestamp structure.
#[repr(C)]
pub struct RtcTime {
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u16,
}

/// Print an integer formatted to at least 2 digits with a leading zero if needed.
pub fn print_2digit(n: u64) {
    if n < 10 {
        vga::print_str("0");
    }
    vga::print_u64(n);
}

/// Probe and count connected PCI configuration space devices across all buses and slots.
pub fn count_pci_devices() -> u64 {
    #[cfg(not(target_os = "none"))]
    {
        0
    }
    #[cfg(target_os = "none")]
    {
        let mut count = 0u64;
        for bus in 0..=255u16 {
            for slot in 0..32u8 {
                let address = ((bus as u32) << 16) | ((slot as u32) << 11) | 0x80000000u32;
                unsafe {
                    core::arch::asm!(
                        "out dx, eax",
                        in("dx") 0xCF8u16,
                        in("eax") address,
                        options(nomem, nostack, preserves_flags)
                    );
                    let value: u32;
                    core::arch::asm!(
                        "in eax, dx",
                        out("eax") value,
                        in("dx") 0xCFCu16,
                        options(nomem, nostack, preserves_flags)
                    );
                    if (value & 0xFFFF) != 0xFFFF {
                        count += 1;
                    }
                }
            }
        }
        count
    }
}

}

pub use self::hardware::*;
pub mod router {
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
        "memory" | "mem" => crate::cmds::memory::run(&mut parts),
        "slabinfo" | "slab" => crate::cmds::slabinfo::run(&mut parts),
        "devices" | "pci" => crate::cmds::devices::run(&mut parts),
        "drivers" => crate::cmds::drivers::run(&mut parts),
        "regs" => crate::cmds::sys::info::regs::run(&mut parts),
        "irq" => crate::cmds::sys::info::irq::run(&mut parts),
        "disk" => crate::cmds::disk::run(&mut parts),
        "dcache" => crate::cmds::dcache::run(&mut parts),
        "icache" => crate::cmds::icache::run(&mut parts),
        "mount" => crate::cmds::fs::mount::mount::run(&mut parts),
        "pipes" => crate::cmds::sys::info::pipes::run(&mut parts),
        "sync" => crate::cmds::sync::run(&mut parts),
        "syslog" | "dmesg" => crate::cmds::syslog::run(&mut parts),
        "unwind" => crate::cmds::unwind::run(&mut parts),
        "watchpoint" => crate::cmds::watchpoint::run(&mut parts),
        "wipe" => crate::cmds::sys::control::wipe::run(&mut parts),
        "clear" => {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str(
                "Unknown command: clear. Type 'help' for available commands or 'wipe' to reset console.\n",
            );
            vga::set_color(vga::Color::White, vga::Color::Black);
        }
        "pidinfo" | "procs" => crate::cmds::proc::pidinfo::run(&mut parts),
        "secinfo" | "mitigations" => crate::cmds::sys::info::secinfo::run(&mut parts),
        "aslrinfo" | "aslr" => crate::cmds::sys::info::aslrinfo::run(&mut parts),
        "rnginfo" | "entropy" => crate::cmds::sys::info::rnginfo::run(&mut parts),
        "rlimit" | "limit" => crate::cmds::sys::info::rlimit::run(&mut parts),
        "unixsock" | "unix" => crate::cmds::sys::info::unixsock::run(&mut parts),
        "vmainfo" | "vma" => crate::cmds::sys::info::vmainfo::run(&mut parts),
        "pollinfo" | "poll" => crate::cmds::sys::info::pollinfo::run(&mut parts),
        "selectinfo" | "select" => crate::cmds::sys::info::selectinfo::run(&mut parts),
        "run" => crate::cmds::run::run(&mut parts),
        "sh" => {
            let sh_path = if keira_fs::vfs::exists("/bin/sh.elf") {
                "/bin/sh.elf"
            } else if keira_fs::vfs::exists("/bin/sh") {
                "/bin/sh"
            } else {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("sh: executable not found in /bin\n");
                vga::set_color(vga::Color::White, vga::Color::Black);
                return;
            };

            unsafe {
                let _ = crate::cmds::proc::task::run::run_user_program(sh_path, &[sh_path]);
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("\n[kernel] Userspace session terminated.\n");
                vga::print_str(
                    "[kernel] Dropping to supervisor console (Ring 0). Type 'help' or 'sh'.\n\n",
                );
                vga::set_color(vga::Color::White, vga::Color::Black);
            }
        }
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
                vga::set_color(vga::Color::White, vga::Color::Black);
            } else {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("Unknown command: ");
                vga::print_str(command);
                vga::print_str(". Type 'help' for available commands.\n");
                vga::set_color(vga::Color::White, vga::Color::Black);
            }
        }
    }
}

}

pub use self::router::*;
