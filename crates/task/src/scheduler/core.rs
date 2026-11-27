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

//! Core scheduler state and execution lifecycle hooks.

pub mod hooks {
    use core::fmt::Write;

    use super::state::{CURRENT_TASK_IDX, TASKS};
    use crate::types::{TaskState, MAX_TASKS};

    pub type TaskResourceCleanupHook = fn(pid: usize);
    pub static mut TASK_CLEANUP_HOOK: Option<TaskResourceCleanupHook> = None;

    pub fn register_task_cleanup_hook(hook: TaskResourceCleanupHook) {
        unsafe {
            TASK_CLEANUP_HOOK = Some(hook);
        }
    }

    struct SliceWriter<'a> {
        buf: &'a mut [u8],
        offset: usize,
    }

    impl<'a> Write for SliceWriter<'a> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            let avail = self.buf.len().saturating_sub(self.offset);
            let to_write = bytes.len().min(avail);
            self.buf[self.offset..self.offset + to_write].copy_from_slice(&bytes[..to_write]);
            self.offset += to_write;
            Ok(())
        }
    }

    pub fn task_status_provider(pid: usize, buf: &mut [u8]) -> Option<usize> {
        unsafe {
            if pid >= MAX_TASKS {
                return None;
            }
            let task = TASKS[pid].as_ref()?;
            let mut open_fds = 0;
            for fd in &task.fds {
                if fd.is_open {
                    open_fds += 1;
                }
            }
            let state_str = match task.state {
                TaskState::Running => "R (running)",
                TaskState::Ready => "S (sleeping)",
                TaskState::Blocked => "D (disk sleep)",
                TaskState::Zombie(_) | TaskState::Exited(_) => "Z (zombie)",
                TaskState::Created => "I (idle)",
            };
            let vsize_kb = if task.program_break > task.program_break_start {
                (task.program_break.saturating_sub(task.program_break_start)
                    + super::state::TASK_STACK_SIZE)
                    / 1024
            } else {
                super::state::TASK_STACK_SIZE / 1024
            };
            let rss_kb = vsize_kb;

            let mut writer = SliceWriter { buf, offset: 0 };
            let _ = core::write!(
                writer,
                "Name:\t{}\n\
                 State:\t{}\n\
                 Tgid:\t{}\n\
                 Pid:\t{}\n\
                 PPid:\t{}\n\
                 Uid:\t{}\t{}\n\
                 Gid:\t{}\t{}\n\
                 FDSize:\t{}\n\
                 VmSize:\t{} kB\n\
                 VmRSS:\t{} kB\n\
                 Threads:\t1\n\
                 SigBlk:\t{:08x}\n\
                 SigPnd:\t{:08x}\n\
                 voluntary_ctxt_switches:\t{}\n",
                task.comm_str(),
                state_str,
                task.id,
                task.id,
                task.parent_id,
                task.uid,
                task.euid,
                task.gid,
                task.egid,
                open_fds,
                vsize_kb,
                rss_kb,
                task.signal_mask,
                task.pending_signals,
                task.switches
            );
            Some(writer.offset)
        }
    }

    pub fn task_cmdline_provider(pid: usize, buf: &mut [u8]) -> Option<usize> {
        unsafe {
            if pid >= MAX_TASKS {
                return None;
            }
            let task = TASKS[pid].as_ref()?;
            if !task.is_user && task.id != 0 {
                return Some(0);
            }
            let bytes = task.cmdline_bytes();
            let to_copy = bytes.len().min(buf.len());
            buf[..to_copy].copy_from_slice(&bytes[..to_copy]);
            Some(to_copy)
        }
    }

    pub fn task_stat_provider(pid: usize, buf: &mut [u8]) -> Option<usize> {
        unsafe {
            if pid >= MAX_TASKS {
                return None;
            }
            let task = TASKS[pid].as_ref()?;
            let state_char = match task.state {
                TaskState::Running => 'R',
                TaskState::Ready => 'S',
                TaskState::Blocked => 'D',
                TaskState::Zombie(_) | TaskState::Exited(_) => 'Z',
                TaskState::Created => 'I',
            };
            let pgrp = if task.tgid != 0 { task.tgid } else { task.id };
            let vsize = if task.program_break > task.program_break_start {
                task.program_break.saturating_sub(task.program_break_start)
                    + super::state::TASK_STACK_SIZE
            } else {
                super::state::TASK_STACK_SIZE
            };
            let rss = vsize.div_ceil(4096);

            let mut writer = SliceWriter { buf, offset: 0 };
            let _ = writeln!(
                writer,
                "{} ({}) {} {} {} 0 0 -1 4194304 0 0 0 0 {} {} 0 0 20 0 1 0 0 {} {} 18446744073709551615 4194304 8388608 {} {} 0 {} {} 0 0 0 0 0 17 0 0 0 0 0 0 {} {} {} 0 0 0 0 {}",
                task.id,
                task.comm_str(),
                state_char,
                task.parent_id,
                pgrp,
                task.cpu_ticks,
                task.switches,
                vsize,
                rss,
                task.rsp,
                task.rsp,
                task.pending_signals,
                task.signal_mask,
                task.program_break_start,
                task.program_break,
                task.program_break_start,
                task.exit_code
            );
            Some(writer.offset)
        }
    }

    pub fn task_statm_provider(pid: usize, buf: &mut [u8]) -> Option<usize> {
        unsafe {
            if pid >= MAX_TASKS {
                return None;
            }
            let task = TASKS[pid].as_ref()?;
            let vsize = if task.program_break > task.program_break_start {
                task.program_break.saturating_sub(task.program_break_start)
                    + super::state::TASK_STACK_SIZE
            } else {
                super::state::TASK_STACK_SIZE
            };
            let total_pages = vsize.div_ceil(4096);
            let rss_pages = total_pages;
            let shared_pages = 0;
            let text_pages = if task.is_user { 4 } else { 1 };
            let lib_pages = 0;
            let data_pages = task
                .program_break
                .saturating_sub(task.program_break_start)
                .div_ceil(4096);
            let dt_pages = 0;

            let mut writer = SliceWriter { buf, offset: 0 };
            let _ = writeln!(
                writer,
                "{} {} {} {} {} {} {}",
                total_pages, rss_pages, shared_pages, text_pages, lib_pages, data_pages, dt_pages
            );
            Some(writer.offset)
        }
    }

    pub fn task_exists_provider(pid: usize) -> bool {
        unsafe {
            if pid >= MAX_TASKS {
                return false;
            }
            TASKS[pid].is_some()
        }
    }

    pub fn task_list_provider(out_pids: &mut [usize]) -> usize {
        unsafe {
            let mut count = 0;
            for (i, task) in TASKS.iter().enumerate() {
                if task.is_some() && count < out_pids.len() {
                    out_pids[count] = i;
                    count += 1;
                }
            }
            count
        }
    }

    pub fn current_pid_provider() -> usize {
        unsafe { CURRENT_TASK_IDX }
    }

    pub fn task_limits_provider(pid: usize, buf: &mut [u8]) -> Option<usize> {
        unsafe {
            if pid >= MAX_TASKS {
                return None;
            }
            let task = TASKS[pid].as_ref()?;
            Some(crate::rlimit::format_task_limits(&task.rlimits, buf))
        }
    }

    fn write_vaddr_range(writer: &mut SliceWriter, start: u64, end: u64) {
        if end <= 0xFFFF_FFFF {
            let _ = core::write!(writer, "{:08x}-{:08x}", start, end);
        } else {
            let _ = core::write!(writer, "{:012x}-{:012x}", start, end);
        }
    }

    pub fn task_maps_provider(pid: usize, buf: &mut [u8]) -> Option<usize> {
        unsafe {
            if pid >= MAX_TASKS {
                return None;
            }
            let task = TASKS[pid].as_ref()?;
            let mut writer = SliceWriter { buf, offset: 0 };

            if !task.is_user {
                // Kernel thread: emit kernel stack mapping
                let stack_top = task.stack_addr + super::state::TASK_STACK_SIZE;
                write_vaddr_range(&mut writer, task.stack_addr, stack_top);
                let _ = writeln!(writer, " rw-p 00000000 00:00 0                  [kstack]");
                return Some(writer.offset);
            }

            // 1. Text segment (ELF executable)
            let text_start: u64 = 0x0040_0000;
            let text_end: u64 = text_start + 0x5000;
            write_vaddr_range(&mut writer, text_start, text_end);
            let _ = writeln!(
                writer,
                " r-xp 00000000 00:00 0                  {}",
                task.comm_str()
            );

            // 2. Data / BSS segment
            let data_start: u64 = text_end;
            let data_end: u64 = data_start + 0x1000;
            write_vaddr_range(&mut writer, data_start, data_end);
            let _ = writeln!(
                writer,
                " rw-p 00005000 00:00 0                  {}",
                task.comm_str()
            );

            // 3. Heap segment (if active)
            if task.program_break > task.program_break_start {
                write_vaddr_range(&mut writer, task.program_break_start, task.program_break);
                let _ = writeln!(writer, " rw-p 00000000 00:00 0                  [heap]");
            }

            // 4. Dynamic VMAs from VMA_TABLE (anonymous mmap or file-backed mmap)
            #[cfg(target_arch = "x86")]
            let has_vma_root = true;
            #[cfg(not(target_arch = "x86"))]
            let has_vma_root = task.pml4_phys != 0;

            if has_vma_root {
                let mut vmas = [keira_mem::Vma::empty(); keira_mem::MAX_VMAS];
                let count = keira_mem::get_vmas_for_pml4(task.pml4_phys, &mut vmas);
                for vma in &vmas[..count] {
                    write_vaddr_range(&mut writer, vma.start, vma.end);
                    let r = if (vma.prot & keira_mem::PROT_READ) != 0 {
                        'r'
                    } else {
                        '-'
                    };
                    let w = if (vma.prot & keira_mem::PROT_WRITE) != 0 {
                        'w'
                    } else {
                        '-'
                    };
                    let x = if (vma.prot & keira_mem::PROT_EXEC) != 0 {
                        'x'
                    } else {
                        '-'
                    };
                    let p = if (vma.flags & keira_mem::MAP_SHARED) != 0 {
                        's'
                    } else {
                        'p'
                    };
                    let path = vma.file_path_str().unwrap_or("[anon]");
                    let _ = writeln!(
                        writer,
                        " {}{}{}{} {:08x} 00:00 0                  {}",
                        r, w, x, p, vma.file_offset, path
                    );
                }
            }

            // 5. User Stack segment
            #[cfg(target_arch = "x86")]
            {
                let stack_top = 0x07FFF000u64;
                let stack_bottom = stack_top.saturating_sub(0x20000);
                write_vaddr_range(&mut writer, stack_bottom, stack_top);
                let _ = core::write!(writer, " rw-p 00000000 00:00 0                  [stack]\n");
            }
            #[cfg(not(target_arch = "x86"))]
            {
                let stack_top = 0x7FFFFFE00000u64.saturating_sub(task.stack_aslr_offset);
                let stack_bottom = stack_top.saturating_sub(0x20000);
                write_vaddr_range(&mut writer, stack_bottom, stack_top);
                let _ = writeln!(writer, " rw-p 00000000 00:00 0                  [stack]");
            }

            Some(writer.offset)
        }
    }
}

pub mod state {
    use core::sync::atomic::{AtomicUsize, Ordering};
    use keira_core::sync::{IrqSpinLock, LockRank};
    use keira_mem::{pmm, vmm};

    use super::hooks::{
        current_pid_provider, task_cmdline_provider, task_exists_provider, task_limits_provider,
        task_list_provider, task_maps_provider, task_stat_provider, task_statm_provider,
        task_status_provider,
    };
    pub use crate::types::MAX_TASKS;
    use crate::types::{FileDescriptor, Task, TaskState, MAX_FDS};

    extern "C" {
        pub static mut main_kernel_stack: u64;
        pub fn set_kernel_stack(sp0: usize);
        pub fn get_boot_kernel_stack() -> usize;
    }

    #[cfg(target_arch = "x86_64")]
    pub const TASK_STACK_PAGES: usize = 4;
    #[cfg(not(target_arch = "x86_64"))]
    pub const TASK_STACK_PAGES: usize = 2;

    pub const TASK_STACK_SIZE: u64 = (TASK_STACK_PAGES as u64) * keira_mem::pmm::PAGE_SIZE;

    pub static SCHEDULER_LOCK: IrqSpinLock = IrqSpinLock::with_rank(LockRank::Scheduler);
    pub static mut TASKS: [Option<Task>; MAX_TASKS] = [const { None }; MAX_TASKS];
    pub static mut CURRENT_TASK_IDX: usize = 0;
    pub static mut SCHEDULER_INITIALIZED: bool = false;

    pub static TOTAL_CONTEXT_SWITCHES: AtomicUsize = AtomicUsize::new(0);

    pub static TOTAL_SCHEDULER_TICKS: AtomicUsize = AtomicUsize::new(0);

    pub fn scheduler_get_stats() -> (u64, u64, usize) {
        let switches = TOTAL_CONTEXT_SWITCHES.load(Ordering::Relaxed) as u64;
        let ticks = TOTAL_SCHEDULER_TICKS.load(Ordering::Relaxed) as u64;
        let active = unsafe { TASKS.iter().filter(|t| t.is_some()).count() };
        (switches, ticks, active)
    }

    pub unsafe fn init() {
        let mut main_cwd = [0u8; 128];
        main_cwd[0] = b'/';
        let boot_pml4 = vmm::active_pml4();
        let main_stack = pmm::alloc_contiguous_frames(TASK_STACK_PAGES).unwrap_or(0);
        let mut main_fds = [FileDescriptor::new(); MAX_FDS];
        for (fd, item) in main_fds.iter_mut().enumerate().take(3) {
            item.is_open = true;
            item.write_mode = fd != 0;
            item.path[..8].copy_from_slice(b"/dev/tty");
            item.path_len = 8;
        }
        let main_task = Task {
            id: 0,
            name: "kernel_shell",
            comm: [0u8; 32],
            comm_len: 0,
            cmdline: [0u8; 128],
            cmdline_len: 0,
            rsp: 0,
            stack_addr: main_stack,
            state: TaskState::Running,
            fds: main_fds,
            program_break: 0,
            program_break_start: 0,
            cwd: main_cwd,
            cwd_len: 1,
            parent_id: 0,
            pml4_phys: boot_pml4,
            exit_code: 0,
            is_user: false,
            uid: 0,
            gid: 0,
            euid: 0,
            egid: 0,
            saved_sigcontext: None,
            signal_mask: 0,
            pending_signals: 0,
            is_orphan: false,
            tgid: 0,
            is_thread: false,
            clear_child_tid: 0,
            tls: 0,
            cpu_ticks: 0,
            switches: 1,
            stack_aslr_offset: 0,
            mmap_aslr_offset: 0,
            brk_aslr_offset: 0,
            rlimits: crate::rlimit::default_rlimits(),
            pgid: 0,
            sid: 0,
        };
        for slot in TASKS[1..].iter_mut() {
            *slot = None;
        }
        TASKS[0] = Some(main_task);
        CURRENT_TASK_IDX = 0;
        SCHEDULER_INITIALIZED = true;
        crate::scheduler::kthread::init_kthreads();
        keira_fs::proc::register_task_hooks(
            task_status_provider,
            task_cmdline_provider,
            current_pid_provider,
        );
        keira_fs::proc::register_extended_task_hooks(
            task_stat_provider,
            task_statm_provider,
            task_exists_provider,
            task_list_provider,
        );
        keira_fs::proc::register_task_limits_hook(task_limits_provider);
        keira_fs::proc::register_task_maps_hook(task_maps_provider);
        keira_mem::register_mmap_aslr_hook(mmap_aslr_offset_provider);
        keira_fs::proc::register_aslr_hook(Some(crate::aslr::get_aslr_policy));
        keira_fs::proc::register_waitqueue_hook(crate::wait::format_waitqueue_stats);
        keira_fs::proc::register_workqueues_hook(crate::workqueue::format_workqueue_stats);
    }

    fn mmap_aslr_offset_provider() -> u64 {
        unsafe {
            let idx = CURRENT_TASK_IDX;
            if let Some(ref t) = TASKS[idx] {
                t.mmap_aslr_offset
            } else {
                0
            }
        }
    }
}

pub use self::hooks::*;
pub use self::state::*;
