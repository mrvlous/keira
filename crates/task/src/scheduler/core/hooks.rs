// SPDX-License-Identifier: GPL-2.0-only

//! Task cleanup hooks and procfs dynamic information providers.

use core::fmt::Write;

use super::state::{CURRENT_TASK_IDX, TASKS};
use crate::types::{TaskState, MAX_TASKS};

pub type TaskResourceCleanupHook = fn(pid: usize);
pub static mut TASK_CLEANUP_HOOK: Option<TaskResourceCleanupHook> = None;

/// Register kernel-level resource cleanup callback (invoked on task exit and zombie reaping).
pub fn register_task_cleanup_hook(hook: TaskResourceCleanupHook) {
    unsafe {
        TASK_CLEANUP_HOOK = Some(hook);
    }
}

/// Fixed slice writer helper implementing core::fmt::Write.
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

/// Dynamic task status provider for /proc/[pid]/status.
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
            task.name,
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

/// Dynamic task cmdline provider for /proc/[pid]/cmdline.
pub fn task_cmdline_provider(pid: usize, buf: &mut [u8]) -> Option<usize> {
    unsafe {
        if pid >= MAX_TASKS {
            return None;
        }
        let task = TASKS[pid].as_ref()?;
        let bytes = task.name.as_bytes();
        let to_copy = bytes.len().min(buf.len());
        buf[..to_copy].copy_from_slice(&bytes[..to_copy]);
        Some(to_copy)
    }
}

/// Dynamic task accounting stat provider for /proc/[pid]/stat.
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
        let rss = (vsize + 4095) / 4096;

        let mut writer = SliceWriter { buf, offset: 0 };
        let _ = core::write!(
            writer,
            "{} ({}) {} {} {} 0 0 -1 4194304 0 0 0 0 {} {} 0 0 20 0 1 0 0 {} {} 18446744073709551615 4194304 8388608 {} {} 0 {} {} 0 0 0 0 0 17 0 0 0 0 0 0 {} {} {} 0 0 0 0 {}\n",
            task.id,
            task.name,
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

/// Dynamic task memory page metrics provider for /proc/[pid]/statm.
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
        let total_pages = (vsize + 4095) / 4096;
        let rss_pages = total_pages;
        let shared_pages = 0;
        let text_pages = if task.is_user { 4 } else { 1 };
        let lib_pages = 0;
        let data_pages =
            (task.program_break.saturating_sub(task.program_break_start) + 4095) / 4096;
        let dt_pages = 0;

        let mut writer = SliceWriter { buf, offset: 0 };
        let _ = core::write!(
            writer,
            "{} {} {} {} {} {} {}\n",
            total_pages,
            rss_pages,
            shared_pages,
            text_pages,
            lib_pages,
            data_pages,
            dt_pages
        );
        Some(writer.offset)
    }
}

/// Checks whether a task exists in the global scheduler table.
pub fn task_exists_provider(pid: usize) -> bool {
    unsafe {
        if pid >= MAX_TASKS {
            return false;
        }
        TASKS[pid].is_some()
    }
}

/// Enumerates all active task PIDs into the provided buffer.
pub fn task_list_provider(out_pids: &mut [usize]) -> usize {
    unsafe {
        let mut count = 0;
        for (i, task) in TASKS.iter().enumerate() {
            if task.is_some() {
                if count < out_pids.len() {
                    out_pids[count] = i;
                    count += 1;
                }
            }
        }
        count
    }
}

/// Dynamic provider for active task PID.
pub fn current_pid_provider() -> usize {
    unsafe { CURRENT_TASK_IDX }
}

/// Dynamic task resource limits provider for /proc/[pid]/limits.
pub fn task_limits_provider(pid: usize, buf: &mut [u8]) -> Option<usize> {
    unsafe {
        if pid >= MAX_TASKS {
            return None;
        }
        let task = TASKS[pid].as_ref()?;
        Some(crate::rlimit::format_task_limits(&task.rlimits, buf))
    }
}

/// Helper to write virtual address range into maps output.
fn write_vaddr_range(writer: &mut SliceWriter, start: u64, end: u64) {
    if end <= 0xFFFF_FFFF {
        let _ = core::write!(writer, "{:08x}-{:08x}", start, end);
    } else {
        let _ = core::write!(writer, "{:012x}-{:012x}", start, end);
    }
}

/// Dynamic task virtual memory mappings provider for /proc/[pid]/maps.
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
            let _ = core::write!(writer, " rw-p 00000000 00:00 0                  [kstack]\n");
            return Some(writer.offset);
        }

        // 1. Text segment (ELF executable)
        let text_start: u64 = 0x0040_0000;
        let text_end: u64 = text_start + 0x5000;
        write_vaddr_range(&mut writer, text_start, text_end);
        let _ = core::write!(
            writer,
            " r-xp 00000000 00:00 0                  {}\n",
            task.name
        );

        // 2. Data / BSS segment
        let data_start: u64 = text_end;
        let data_end: u64 = data_start + 0x1000;
        write_vaddr_range(&mut writer, data_start, data_end);
        let _ = core::write!(
            writer,
            " rw-p 00005000 00:00 0                  {}\n",
            task.name
        );

        // 3. Heap segment (if active)
        if task.program_break > task.program_break_start {
            write_vaddr_range(&mut writer, task.program_break_start, task.program_break);
            let _ = core::write!(writer, " rw-p 00000000 00:00 0                  [heap]\n");
        }

        // 4. Dynamic VMAs from VMA_TABLE (anonymous mmap or file-backed mmap)
        if task.pml4_phys != 0 {
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
                let path = if let Some(p) = vma.file_path_str() {
                    p
                } else {
                    "[anon]"
                };
                let _ = core::write!(
                    writer,
                    " {}{}{}{} {:08x} 00:00 0                  {}\n",
                    r,
                    w,
                    x,
                    p,
                    vma.file_offset,
                    path
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
            let _ = core::write!(writer, " rw-p 00000000 00:00 0                  [stack]\n");
        }

        Some(writer.offset)
    }
}
