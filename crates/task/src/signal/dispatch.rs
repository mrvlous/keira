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

//! POSIX signal dispatch and kill syscall implementation.

pub mod kill {
    use keira_io::vga;

    use crate::signal::job::{JobState, JOB_COUNT, JOB_TABLE};
    use crate::signal::posix::*;

    pub fn sys_kill(pid: u32, sig: u32) -> Result<u64, &'static str> {
        unsafe {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("[SIGNAL] Dispatched POSIX Signal ");
            vga::print_u64(sig as u64);
            vga::print_str(" -> PID ");
            vga::print_u64(pid as u64);
            vga::print_str(" (Syscall 22)\n");

            for job in JOB_TABLE.iter_mut().take(JOB_COUNT).flatten() {
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
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            crate::scheduler::send_signal(pid as usize, sig)?;
        }
        Ok(0)
    }
}

pub use self::kill::*;
