// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Process kill signal delivery dispatch.

use keira_io::vga;

use crate::signal::job::{JobState, JOB_COUNT, JOB_TABLE};
use crate::signal::posix::*;

/// Send POSIX signal to target process PID (Syscall 22: sys_kill).
pub fn sys_kill(pid: u32, sig: u32) -> Result<u64, &'static str> {
    unsafe {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("[SIGNAL] Dispatched POSIX Signal ");
        vga::print_u64(sig as u64);
        vga::print_str(" -> PID ");
        vga::print_u64(pid as u64);
        vga::print_str(" (Syscall 22)\n");

        for i in 0..JOB_COUNT {
            if let Some(ref mut job) = JOB_TABLE[i] {
                if job.pid == pid {
                    match sig {
                        SIGKILL | SIGTERM | SIGINT | SIGQUIT | SIGABRT | SIGSEGV | SIGILL
                        | SIGBUS | SIGFPE | SIGPIPE | SIGHUP | SIGUSR1 | SIGUSR2 | SIGALRM => {
                            job.state = JobState::Terminated;
                        }
                        SIGSTOP => {
                            job.state = JobState::Stopped;
                        }
                        SIGCONT => {
                            job.state = JobState::Running;
                        }
                        _ => {}
                    }
                }
            }
        }
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);

        crate::scheduler::send_signal(pid as usize, sig)?;
    }
    Ok(0)
}
