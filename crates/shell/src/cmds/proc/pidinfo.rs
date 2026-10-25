// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'pidinfo' supervisor diagnostic shell command.

use keira_io::vga;
use keira_task::scheduler::TASKS;
use keira_task::types::{TaskState, MAX_FDS, MAX_TASKS};

/// Executes the 'pidinfo' supervisor diagnostic command.
pub fn run(parts: &mut core::str::SplitWhitespace) {
    let bg = vga::Color::Black;
    let fg = vga::Color::White;

    if let Some(arg) = parts.next() {
        if arg == "-h" || arg == "--help" {
            vga::set_color(fg, bg);
            vga::print_str("Usage: pidinfo [-h] [pid]\n\n");
            vga::print_str(
                "Description:\n  Inspect active process hierarchy, memory metrics and kernel task descriptors.\n\n",
            );
            vga::print_str("Options:\n");
            vga::print_str("  [pid]         Inspect specific process by numerical identifier\n");
            vga::print_str("  -h, --help    Show this help reference and exit\n");
            vga::set_color(vga::Color::LightGrey, bg);
            return;
        }

        if let Ok(pid) = parse_usize(arg) {
            inspect_single_pid(pid, fg, bg);
            return;
        }

        vga::set_color(vga::Color::LightRed, bg);
        vga::print_str("Error: Invalid process identifier '");
        vga::print_str(arg);
        vga::print_str("'\n");
        vga::set_color(vga::Color::LightGrey, bg);
        return;
    }

    list_all_tasks(fg, bg);
}

/// Lists all active processes in a concise summary table.
fn list_all_tasks(fg: vga::Color, bg: vga::Color) {
    vga::set_color(fg, bg);
    vga::print_str("Active Process Table:\n");
    vga::print_str("  PID   PPID  STATE        NAME             VMSIZE   RSS     SWITCHES\n");

    unsafe {
        let mut found = 0usize;
        for (i, slot) in TASKS.iter().enumerate() {
            if let Some(ref task) = slot {
                found += 1;
                vga::print_str("  ");
                print_padded_num(task.id as u64, 5);
                vga::print_str(" ");
                print_padded_num(task.parent_id as u64, 5);
                vga::print_str(" ");

                let state_str = match task.state {
                    TaskState::Running => "Running     ",
                    TaskState::Ready => "Sleeping    ",
                    TaskState::Blocked => "DiskSleep   ",
                    TaskState::Zombie(_) | TaskState::Exited(_) => "Zombie      ",
                    TaskState::Created => "Idle        ",
                };
                vga::print_str(state_str);
                vga::print_str(" ");

                print_padded_str(task.name, 16);
                vga::print_str(" ");

                let vsize_kb = if task.program_break > task.program_break_start {
                    (task.program_break.saturating_sub(task.program_break_start) + 16384) / 1024
                } else {
                    16
                };
                print_padded_num(vsize_kb, 6);
                vga::print_str("kB  ");

                print_padded_num(vsize_kb, 5);
                vga::print_str("kB  ");

                vga::print_u64(task.switches);
                vga::print_str("\n");
            }
            if i >= MAX_TASKS {
                break;
            }
        }

        vga::print_str("\nTotal active tasks: ");
        vga::print_u64(found as u64);
        vga::print_str("\n");
    }

    vga::set_color(vga::Color::LightGrey, bg);
}

/// Inspects detailed kernel descriptors for a specific process ID.
fn inspect_single_pid(pid: usize, fg: vga::Color, bg: vga::Color) {
    if pid >= MAX_TASKS {
        vga::set_color(vga::Color::LightRed, bg);
        vga::print_str("Error: Process ID exceeds maximum task capacity.\n");
        vga::set_color(vga::Color::LightGrey, bg);
        return;
    }

    unsafe {
        if let Some(ref task) = TASKS[pid] {
            vga::set_color(fg, bg);
            vga::print_str("Process Information (PID ");
            vga::print_u64(task.id as u64);
            vga::print_str("):\n");

            vga::print_str("  Command Name:     ");
            vga::print_str(task.name);
            vga::print_str("\n");

            vga::print_str("  Process State:    ");
            let state_desc = match task.state {
                TaskState::Running => "Running on CPU",
                TaskState::Ready => "Sleeping (Runnable)",
                TaskState::Blocked => "Blocked on I/O or sleep timer",
                TaskState::Zombie(code) => {
                    vga::print_str("Zombie (exit code: ");
                    vga::print_u64(code as u64);
                    vga::print_str(")");
                    ""
                }
                TaskState::Exited(code) => {
                    vga::print_str("Exited (code: ");
                    vga::print_u64(code as u64);
                    vga::print_str(")");
                    ""
                }
                TaskState::Created => "Created (not scheduled)",
            };
            if !state_desc.is_empty() {
                vga::print_str(state_desc);
            }
            vga::print_str("\n");

            vga::print_str("  Thread Group ID:  ");
            vga::print_u64(task.tgid as u64);
            vga::print_str("\n");

            vga::print_str("  Parent PID:       ");
            vga::print_u64(task.parent_id as u64);
            vga::print_str("\n");

            vga::print_str("  Credentials:      UID: ");
            vga::print_u64(task.uid as u64);
            vga::print_str(", GID: ");
            vga::print_u64(task.gid as u64);
            vga::print_str(" (euid: ");
            vga::print_u64(task.euid as u64);
            vga::print_str(", egid: ");
            vga::print_u64(task.egid as u64);
            vga::print_str(")\n");

            let vsize_kb = if task.program_break > task.program_break_start {
                (task.program_break.saturating_sub(task.program_break_start) + 16384) / 1024
            } else {
                16
            };
            vga::print_str("  Virtual Memory:   ");
            vga::print_u64(vsize_kb);
            vga::print_str(" kB (RSS: ");
            vga::print_u64(vsize_kb);
            vga::print_str(" kB)\n");

            vga::print_str("  Program Break:    ");
            vga::print_hex(task.program_break_start);
            vga::print_str(" -> ");
            vga::print_hex(task.program_break);
            vga::print_str("\n");

            let mut open_fds = 0usize;
            for fd in 0..MAX_FDS {
                if task.fds[fd].is_open {
                    open_fds += 1;
                }
            }
            vga::print_str("  Open Descriptors: ");
            vga::print_u64(open_fds as u64);
            vga::print_str(" / ");
            vga::print_u64(MAX_FDS as u64);
            vga::print_str("\n");

            vga::print_str("  CPU Context:      Ticks: ");
            vga::print_u64(task.cpu_ticks);
            vga::print_str(", Switches: ");
            vga::print_u64(task.switches);
            vga::print_str("\n");

            vga::print_str("  Signal Bitmask:   Mask: ");
            vga::print_hex(task.signal_mask as u64);
            vga::print_str(", Pending: ");
            vga::print_hex(task.pending_signals as u64);
            vga::print_str("\n");

            vga::set_color(vga::Color::LightGrey, bg);
            return;
        }
    }

    vga::set_color(vga::Color::LightRed, bg);
    vga::print_str("Error: Process ID ");
    vga::print_u64(pid as u64);
    vga::print_str(" not found in active task table.\n");
    vga::set_color(vga::Color::LightGrey, bg);
}

fn parse_usize(s: &str) -> Result<usize, ()> {
    let mut val = 0usize;
    if s.is_empty() {
        return Err(());
    }
    for b in s.bytes() {
        if !b.is_ascii_digit() {
            return Err(());
        }
        val = val.checked_mul(10).ok_or(())?;
        val = val.checked_add((b - b'0') as usize).ok_or(())?;
    }
    Ok(val)
}

fn print_padded_num(val: u64, width: usize) {
    let mut digits = 1usize;
    let mut tmp = val;
    while tmp >= 10 {
        tmp /= 10;
        digits += 1;
    }
    let pad = width.saturating_sub(digits);
    for _ in 0..pad {
        vga::print_str(" ");
    }
    vga::print_u64(val);
}

fn print_padded_str(s: &str, width: usize) {
    let len = s.len();
    if len > width {
        vga::print_str(&s[..width]);
    } else {
        vga::print_str(s);
        for _ in 0..width.saturating_sub(len) {
            vga::print_str(" ");
        }
    }
}
