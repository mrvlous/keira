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
    vga::print_str("PID    TASK NAME             STATE\n");
    vga::set_color(vga::Color::White, vga::Color::Black);
    for i in 0..MAX_TASKS {
        if let Some(ref task) = TASKS[i] {
            vga::print_u64(task.id as u64);
            let mut pid_len = 0;
            let mut temp = task.id;
            if temp == 0 {
                pid_len = 1;
            } else {
                while temp > 0 {
                    pid_len += 1;
                    temp /= 10;
                }
            }
            for _ in 0..(7 - pid_len) {
                vga::print_str(" ");
            }

            vga::print_str(task.name);
            for _ in 0..(22 - task.name.len()) {
                vga::print_str(" ");
            }

            match task.state {
                TaskState::Created => {
                    vga::set_color(vga::Color::Yellow, vga::Color::Black);
                    vga::print_str("CREATED\n");
                }
                TaskState::Running => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("RUNNING\n");
                }
                TaskState::Ready => {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("READY\n");
                }
                TaskState::Blocked => {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("BLOCKED\n");
                }
                TaskState::Exited(c) | TaskState::Zombie(c) => {
                    vga::set_color(vga::Color::Red, vga::Color::Black);
                    vga::print_str("ZOMBIE (exit ");
                    vga::print_u64(c as u64);
                    vga::print_str(")\n");
                }
            }
        }
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
