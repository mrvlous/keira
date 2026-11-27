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

//! Task descriptors and execution state machines.

pub mod descriptor {
    use super::state::TaskState;
    use crate::types::context::InterruptContext;
    use crate::types::fd::{FileDescriptor, MAX_FDS};

    pub const MAX_TASKS: usize = 64;

    pub struct Task {
        pub id: usize,
        pub name: &'static str,
        pub comm: [u8; 32],
        pub comm_len: usize,
        pub cmdline: [u8; 128],
        pub cmdline_len: usize,
        pub rsp: u64,
        pub stack_addr: u64,
        pub state: TaskState,
        pub fds: [FileDescriptor; MAX_FDS],
        pub program_break: u64,
        pub program_break_start: u64,
        pub cwd: [u8; 128],
        pub cwd_len: usize,
        pub parent_id: usize,
        pub pml4_phys: u64,
        pub exit_code: i32,
        pub is_user: bool,
        pub uid: u32,
        pub gid: u32,
        pub euid: u32,
        pub egid: u32,
        pub saved_sigcontext: Option<InterruptContext>,
        pub signal_mask: u32,
        pub pending_signals: u32,
        pub is_orphan: bool,
        pub tgid: usize,
        pub is_thread: bool,
        pub clear_child_tid: u64,
        pub tls: u64,
        pub cpu_ticks: u64,
        pub switches: u64,
        pub stack_aslr_offset: u64,
        pub mmap_aslr_offset: u64,
        pub brk_aslr_offset: u64,
        pub rlimits: [crate::rlimit::RLimit; crate::rlimit::RLIM_NLIMITS],
        pub pgid: usize,
        pub sid: usize,
    }

    impl Task {
        /// Records the short executable basename (up to 32 bytes) for process reporting.
        pub fn set_comm(&mut self, path: &str) {
            let base = path.rsplit('/').next().unwrap_or(path);
            let bytes = base.as_bytes();
            let len = bytes.len().min(self.comm.len());
            self.comm = [0u8; 32];
            self.comm[..len].copy_from_slice(&bytes[..len]);
            self.comm_len = len;
        }

        /// Records formatted execution arguments for `/proc/<pid>/cmdline`.
        pub fn set_cmdline(&mut self, args: &[&str]) {
            self.cmdline = [0u8; 128];
            let mut pos = 0usize;
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    if pos >= self.cmdline.len() {
                        break;
                    }
                    self.cmdline[pos] = b' ';
                    pos += 1;
                }
                let bytes = arg.as_bytes();
                let n = bytes.len().min(self.cmdline.len() - pos);
                self.cmdline[pos..pos + n].copy_from_slice(&bytes[..n]);
                pos += n;
            }
            self.cmdline_len = pos;
        }

        /// Returns the task's executable command name, falling back to its static descriptor name.
        pub fn comm_str(&self) -> &str {
            if self.comm_len > 0 {
                core::str::from_utf8(&self.comm[..self.comm_len]).unwrap_or(self.name)
            } else {
                self.name
            }
        }

        /// Returns command-line bytes for `/proc/<pid>/cmdline`.
        pub fn cmdline_bytes(&self) -> &[u8] {
            if self.cmdline_len > 0 {
                &self.cmdline[..self.cmdline_len]
            } else {
                self.comm_str().as_bytes()
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_task_comm_and_cmdline_helpers() {
            let mut task = Task {
                id: 5,
                name: "user_app",
                comm: [0u8; 32],
                comm_len: 0,
                cmdline: [0u8; 128],
                cmdline_len: 0,
                rsp: 0,
                stack_addr: 0,
                state: TaskState::Ready,
                fds: [FileDescriptor::new(); MAX_FDS],
                program_break: 0,
                program_break_start: 0,
                cwd: [0u8; 128],
                cwd_len: 1,
                parent_id: 0,
                pml4_phys: 0,
                exit_code: 0,
                is_user: true,
                uid: 0,
                gid: 0,
                euid: 0,
                egid: 0,
                saved_sigcontext: None,
                signal_mask: 0,
                pending_signals: 0,
                is_orphan: false,
                tgid: 5,
                is_thread: false,
                clear_child_tid: 0,
                tls: 0,
                cpu_ticks: 0,
                switches: 0,
                stack_aslr_offset: 0,
                mmap_aslr_offset: 0,
                brk_aslr_offset: 0,
                rlimits: crate::rlimit::default_rlimits(),
                pgid: 5,
                sid: 5,
            };
            assert_eq!(task.comm_str(), "user_app");
            task.set_comm("/bin/curl.elf");
            assert_eq!(task.comm_str(), "curl.elf");
            task.set_cmdline(&["/bin/curl.elf", "http://icanhazip.com"]);
            assert_eq!(task.cmdline_bytes(), b"/bin/curl.elf http://icanhazip.com");
        }
    }
}

pub mod state {
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub enum TaskState {
        /// Newly allocated task context, not yet eligible for CPU time slice.
        Created,
        /// Runnable task waiting in scheduler runqueue.
        Ready,
        /// Currently executing task on CPU.
        Running,
        /// Task blocked waiting on I/O, child exit or sleep timer.
        Blocked,
        /// Terminated task with exit code.
        Exited(i32),
        /// Deceased process awaiting parent reclamation (waitpid) with exit code.
        Zombie(i32),
    }
}

pub use self::descriptor::*;
pub use self::state::*;
