// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! POSIX signal generation, delivery, and process termination handling.

use keira_io::vga;

pub const SIGHUP: u32 = 1;
pub const SIGINT: u32 = 2;
pub const SIGQUIT: u32 = 3;
pub const SIGILL: u32 = 4;
pub const SIGTRAP: u32 = 5;
pub const SIGABRT: u32 = 6;
pub const SIGBUS: u32 = 7;
pub const SIGFPE: u32 = 8;
pub const SIGKILL: u32 = 9;
pub const SIGUSR1: u32 = 10;
pub const SIGSEGV: u32 = 11;
pub const SIGUSR2: u32 = 12;
pub const SIGPIPE: u32 = 13;
pub const SIGALRM: u32 = 14;
pub const SIGTERM: u32 = 15;
pub const SIGCHLD: u32 = 17;
pub const SIGCONT: u32 = 18;
pub const SIGSTOP: u32 = 19;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    Running,
    Stopped,
    Terminated,
}

#[derive(Debug, Clone, Copy)]
pub struct JobInfo {
    pub job_id: u32,
    pub pid: u32,
    pub name: [u8; 32],
    pub name_len: usize,
    pub state: JobState,
    pub is_foreground: bool,
}

pub static mut JOB_TABLE: [Option<JobInfo>; 16] = [None; 16];
pub static mut JOB_COUNT: usize = 0;

/// Register a new background or foreground process job into Job Control Table.
pub unsafe fn add_job(pid: u32, name: &str, is_fg: bool) -> u32 {
    let job_id = (JOB_COUNT + 1) as u32;
    let nbytes = name.as_bytes();
    let len = if nbytes.len() > 32 { 32 } else { nbytes.len() };

    let mut info = JobInfo {
        job_id,
        pid,
        name: [0u8; 32],
        name_len: len,
        state: JobState::Running,
        is_foreground: is_fg,
    };
    info.name[..len].copy_from_slice(&nbytes[..len]);

    if JOB_COUNT < 16 {
        JOB_TABLE[JOB_COUNT] = Some(info);
        JOB_COUNT += 1;
    } else {
        JOB_TABLE[0] = Some(info);
    }
    job_id
}

/// Send POSIX signal to target process PID (Syscall 72: sys_kill).
pub fn sys_kill(pid: u32, sig: u32) -> Result<u64, &'static str> {
    unsafe {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("[SIGNAL] Dispatched POSIX Signal ");
        vga::print_u64(sig as u64);
        vga::print_str(" -> PID ");
        vga::print_u64(pid as u64);
        vga::print_str(" (Syscall 72)\n");

        for i in 0..JOB_COUNT {
            if let Some(ref mut job) = JOB_TABLE[i] {
                if job.pid == pid {
                    match sig {
                        SIGKILL | SIGTERM | SIGINT => {
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
    }
    Ok(0)
}
