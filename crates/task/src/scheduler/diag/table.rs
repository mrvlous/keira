// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Task process table inspection and terminal diagnostics formatting.

use keira_io::{serial, vga};

use crate::scheduler::core::{MAX_TASKS, TASKS};
use crate::types::TaskState;

/// List all registered tasks.
///
/// # Safety
/// Caller must ensure that video output buffer access is synchronized.
pub unsafe fn list_tasks() {
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("PID    TASK NAME             STATE       TICKS       SWITCHES\n");
    vga::print_str("-----  --------------------  ----------  ----------  ----------\n");

    for i in 0..MAX_TASKS {
        if let Some(ref task) = TASKS[i] {
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            print_padded_u64(task.id as u64, 7);

            vga::print_str(task.name);
            for _ in 0..(22usize.saturating_sub(task.name.len())) {
                vga::print_str(" ");
            }

            match task.state {
                TaskState::Created => {
                    vga::set_color(vga::Color::Yellow, vga::Color::Black);
                    vga::print_str("CREATED     ");
                }
                TaskState::Running => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("RUNNING     ");
                }
                TaskState::Ready => {
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str("READY       ");
                }
                TaskState::Blocked => {
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str("BLOCKED     ");
                }
                TaskState::Exited(c) | TaskState::Zombie(c) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("ZOMBIE(");
                    vga::print_u64(c as u64);
                    vga::print_str(")   ");
                }
            }

            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            print_padded_u64(task.cpu_ticks, 12);
            print_padded_u64(task.switches, 10);
            vga::print_str("\n");
        }
    }

    let (switches, ticks, active) = crate::scheduler::core::scheduler_get_stats();
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("\nScheduler Telemetry:\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  Active Tasks          : ");
    vga::print_u64(active as u64);
    vga::print_str("\n  Total Context Switches: ");
    vga::print_u64(switches);
    vga::print_str("\n  Total Scheduler Ticks : ");
    vga::print_u64(ticks);
    vga::print_str("\n");
}

unsafe fn print_padded_u64(val: u64, width: usize) {
    vga::print_u64(val);
    let mut len = 0;
    let mut temp = val;
    if temp == 0 {
        len = 1;
    } else {
        while temp > 0 {
            len += 1;
            temp /= 10;
        }
    }
    for _ in 0..width.saturating_sub(len) {
        vga::print_str(" ");
    }
}

pub(crate) unsafe fn print_decimal(mut val: u64) {
    if val == 0 {
        serial::print_str("0");
        return;
    }
    let mut buf = [0u8; 20];
    let mut idx = 0;
    while val > 0 {
        buf[idx] = b'0' + (val % 10) as u8;
        val /= 10;
        idx += 1;
    }
    while idx > 0 {
        idx -= 1;
        let s = [buf[idx]];
        if let Ok(st) = core::str::from_utf8(&s) {
            serial::print_str(st);
        }
    }
}
