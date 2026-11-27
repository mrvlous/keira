// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

//! Scheduler diagnostic telemetry and task table dumps.

pub mod table {
    use keira_io::vga;

    use crate::scheduler::core::{MAX_TASKS, TASKS};
    use crate::types::TaskState;

    /// Prints the scheduler task table and telemetry counters to the VGA console.
    ///
    /// # Safety
    ///
    /// Reads `static mut TASKS` without acquiring `SCHEDULER_LOCK`. Callers must ensure no
    /// concurrent task creation, state mutation or reaping modifies `TASKS` during the
    /// traversal and that VGA console access is safe in the calling context.
    pub unsafe fn list_tasks() {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("PID    TGID   TASK NAME             STATE       TICKS       SWITCHES\n");
        vga::print_str("-----  -----  --------------------  ----------  ----------  ----------\n");

        for task in TASKS.iter().take(MAX_TASKS).flatten() {
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            print_padded_u64(task.id as u64, 7);
            print_padded_u64(task.tgid as u64, 7);

            let cname = task.comm_str();
            vga::print_str(cname);
            for _ in 0..(22usize.saturating_sub(cname.len())) {
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

    /// Prints a `u64` value followed by trailing spaces up to `width` columns.
    ///
    /// # Safety
    ///
    /// Performs no unsafe operations directly and only calls `vga::print_u64` and
    /// `vga::print_str`. Callers only need to ensure the VGA console is initialized and
    /// synchronized for the calling context.
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
}

pub use self::table::*;
