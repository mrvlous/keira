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

//! Task cloning, process spawning and fork implementation.

pub mod clone {
    #[cfg(not(target_arch = "x86"))]
    use keira_mem::{pmm, vmm};

    #[cfg(not(target_arch = "x86"))]
    use crate::scheduler::core::{
        CURRENT_TASK_IDX, MAX_TASKS, SCHEDULER_LOCK, TASKS, TASK_STACK_PAGES, TASK_STACK_SIZE,
    };
    #[cfg(not(target_arch = "x86"))]
    use crate::scheduler::lifecycle::reap_orphaned_zombies_locked;
    #[cfg(not(target_arch = "x86"))]
    use crate::types::{InterruptContext, Task, TaskState};

    pub const CLONE_VM: u64 = 0x0000_0100;
    pub const CLONE_FS: u64 = 0x0000_0200;
    pub const CLONE_FILES: u64 = 0x0000_0400;
    pub const CLONE_SIGHAND: u64 = 0x0000_0800;
    pub const CLONE_THREAD: u64 = 0x0001_0000;
    pub const CLONE_SETTLS: u64 = 0x0008_0000;
    pub const CLONE_PARENT_SETTID: u64 = 0x0010_0000;
    pub const CLONE_CHILD_CLEARTID: u64 = 0x0020_0000;
    pub const CLONE_CHILD_SETTID: u64 = 0x0100_0000;

    /// Clones the currently running task into a new child process or thread descriptor.
    ///
    /// # Safety
    ///
    /// Must be called with `CURRENT_TASK_IDX < MAX_TASKS` and without holding `SCHEDULER_LOCK`.
    /// On `not(target_arch = "x86")`, callers must invoke this from a valid syscall entry state
    /// where `get_current_percpu()` holds valid saved `user_*` registers for the parent task and
    /// `parent.pml4_phys` is a valid page table root when `CLONE_VM` is unset. Any non-zero,
    /// 4-byte-aligned `ptid` or `ctid` within `0x1000..0x0000_8000_0000_0000` passed with
    /// `CLONE_PARENT_SETTID` or `CLONE_CHILD_SETTID` must be mapped and writable in the active
    /// address space. When `child_stack != 0` or `CLONE_SETTLS` is set, those user addresses must
    /// be valid for the child context.
    pub unsafe fn clone_current_task(
        flags: u64,
        child_stack: u64,
        ptid: u64,
        ctid: u64,
        newtls: u64,
    ) -> Result<usize, &'static str> {
        #[cfg(target_arch = "x86")]
        {
            let _ = (flags, child_stack, ptid, ctid, newtls);
            Err("Scheduler: Fork/Clone is unsupported on 32-bit architecture without MMU paging")
        }

        #[cfg(not(target_arch = "x86"))]
        {
            let _guard = SCHEDULER_LOCK.lock();
            let parent_idx = CURRENT_TASK_IDX;

            let mut slot_idx = 0;
            let mut found = false;
            for (i, slot) in TASKS.iter().enumerate().take(MAX_TASKS).skip(1) {
                if slot.is_none() {
                    slot_idx = i;
                    found = true;
                    break;
                }
            }

            if !found {
                reap_orphaned_zombies_locked();
                for (i, slot) in TASKS.iter().enumerate().take(MAX_TASKS).skip(1) {
                    if slot.is_none() {
                        slot_idx = i;
                        found = true;
                        break;
                    }
                }
            }

            if !found {
                return Err("Scheduler Error: Max tasks reached");
            }

            let ptr = &raw const TASKS;
            if let Some(ref parent) = (*ptr)[parent_idx] {
                let stack_frame = pmm::alloc_contiguous_frames(TASK_STACK_PAGES)
                    .ok_or("Out of memory for child stack")?;
                let stack_top = stack_frame + TASK_STACK_SIZE;

                let pml4_phys = if (flags & CLONE_VM) != 0 {
                    parent.pml4_phys
                } else {
                    match vmm::clone_user_address_space_cow(parent.pml4_phys) {
                        Ok(p) => p,
                        Err(e) => {
                            pmm::free_contiguous_frames(stack_frame, TASK_STACK_PAGES);
                            return Err(e);
                        }
                    }
                };

                let context_size = core::mem::size_of::<InterruptContext>() as u64;
                let child_context_ptr = (stack_top - context_size) as *mut InterruptContext;

                let percpu = keira_arch::cpu::get_current_percpu();

                #[cfg(target_arch = "x86_64")]
                {
                    (*child_context_ptr).r15 = percpu.user_r15;
                    (*child_context_ptr).r14 = percpu.user_r14;
                    (*child_context_ptr).r13 = percpu.user_r13;
                    (*child_context_ptr).r12 = percpu.user_r12;
                    (*child_context_ptr).r11 = 0;
                    (*child_context_ptr).r10 = 0;
                    (*child_context_ptr).r9 = 0;
                    (*child_context_ptr).r8 = 0;
                    (*child_context_ptr).rdi = 0;
                    (*child_context_ptr).rsi = 0;
                    (*child_context_ptr).rbp = percpu.user_rbp;
                    (*child_context_ptr).rbx = percpu.user_rbx;
                    (*child_context_ptr).rdx = 0;
                    (*child_context_ptr).rcx = 0;
                    (*child_context_ptr).rax = 0;

                    let child_rip =
                        if percpu.user_rip >= 0x10000 && percpu.user_rip < 0x0000_8000_0000_0000 {
                            percpu.user_rip
                        } else {
                            0x0000_0000_4000_0000
                        };

                    let child_rsp = if child_stack != 0 {
                        child_stack
                    } else if percpu.user_rsp >= 0x10000 && percpu.user_rsp < 0x0000_8000_0000_0000
                    {
                        percpu.user_rsp
                    } else {
                        0x0000_7FFF_FFFF_F000
                    };

                    (*child_context_ptr).rip = child_rip;
                    (*child_context_ptr).cs = 0x2B;
                    (*child_context_ptr).rflags = (percpu.user_rflags | 0x202) & !0x100;
                    (*child_context_ptr).rsp = child_rsp;
                    (*child_context_ptr).ss = 0x23;
                }

                #[cfg(target_arch = "x86")]
                {
                    (*child_context_ptr).rdi = 0;
                    (*child_context_ptr).rsi = 0;
                    (*child_context_ptr).rbp = percpu.user_rbp as u32;
                    (*child_context_ptr).esp_padding = 0;
                    (*child_context_ptr).rbx = percpu.user_rbx as u32;
                    (*child_context_ptr).rdx = 0;
                    (*child_context_ptr).rcx = 0;
                    (*child_context_ptr).rax = 0;

                    let child_rip = if percpu.user_rip >= 0x10000 && percpu.user_rip < 0xC000_0000 {
                        percpu.user_rip as u32
                    } else {
                        0x0100_0000
                    };

                    let child_rsp = if child_stack != 0 {
                        child_stack as u32
                    } else if percpu.user_rsp >= 0x10000 && percpu.user_rsp < 0xC000_0000 {
                        percpu.user_rsp as u32
                    } else {
                        0x07FF_E000
                    };

                    (*child_context_ptr).rip = child_rip;
                    (*child_context_ptr).cs = 0x1B;
                    (*child_context_ptr).rflags = ((percpu.user_rflags as u32) | 0x202) & !0x100;
                    (*child_context_ptr).rsp = child_rsp;
                    (*child_context_ptr).ss = 0x23;
                }

                let is_thread = (flags & CLONE_THREAD) != 0;
                let tgid = if is_thread { parent.tgid } else { slot_idx };
                let parent_id = if is_thread {
                    parent.parent_id
                } else {
                    parent_idx
                };
                let clear_child_tid = if (flags & CLONE_CHILD_CLEARTID) != 0 {
                    ctid
                } else {
                    0
                };
                let tls = if (flags & CLONE_SETTLS) != 0 {
                    newtls
                } else {
                    parent.tls
                };

                if (flags & CLONE_PARENT_SETTID) != 0
                    && ptid != 0
                    && ptid.is_multiple_of(4)
                    && (0x1000..0x0000_8000_0000_0000).contains(&ptid)
                {
                    let uptr = ptid as *mut u32;
                    core::ptr::write(uptr, slot_idx as u32);
                }

                if (flags & CLONE_CHILD_SETTID) != 0
                    && ctid != 0
                    && ctid.is_multiple_of(4)
                    && (0x1000..0x0000_8000_0000_0000).contains(&ctid)
                {
                    let uptr = ctid as *mut u32;
                    core::ptr::write(uptr, slot_idx as u32);
                }

                let child_task = Task {
                    id: slot_idx,
                    name: if is_thread {
                        "clone_thread"
                    } else {
                        "fork_child"
                    },
                    comm: if is_thread { [0u8; 32] } else { parent.comm },
                    comm_len: if is_thread { 0 } else { parent.comm_len },
                    cmdline: if is_thread {
                        [0u8; 128]
                    } else {
                        parent.cmdline
                    },
                    cmdline_len: if is_thread { 0 } else { parent.cmdline_len },
                    rsp: child_context_ptr as u64,
                    stack_addr: stack_frame,
                    state: TaskState::Ready,
                    fds: parent.fds,
                    program_break: parent.program_break,
                    program_break_start: parent.program_break_start,
                    cwd: parent.cwd,
                    cwd_len: parent.cwd_len,
                    parent_id,
                    pml4_phys,
                    exit_code: 0,
                    is_user: parent.is_user,
                    uid: parent.uid,
                    gid: parent.gid,
                    euid: parent.euid,
                    egid: parent.egid,
                    saved_sigcontext: None,
                    signal_mask: parent.signal_mask,
                    pending_signals: 0,
                    is_orphan: false,
                    tgid,
                    is_thread,
                    clear_child_tid,
                    tls,
                    cpu_ticks: 0,
                    switches: 0,
                    stack_aslr_offset: parent.stack_aslr_offset,
                    mmap_aslr_offset: parent.mmap_aslr_offset,
                    brk_aslr_offset: parent.brk_aslr_offset,
                    rlimits: parent.rlimits,
                    pgid: parent.pgid,
                    sid: parent.sid,
                };

                TASKS[slot_idx] = Some(child_task);
                let core_id = keira_arch::cpu::get_current_core_id();
                crate::scheduler::work_stealing::push_local(core_id, slot_idx);
                Ok(slot_idx)
            } else {
                Err("Scheduler Error: Parent task invalid")
            }
        }
    }
}

pub mod fork {
    use crate::scheduler::process::clone::clone_current_task;

    /// Forks the currently running task using `clone_current_task` with default flags.
    ///
    /// # Safety
    ///
    /// Must be called with `CURRENT_TASK_IDX < MAX_TASKS` and without holding `SCHEDULER_LOCK`.
    /// On `not(target_arch = "x86")`, callers must invoke this from a valid syscall entry state
    /// where `get_current_percpu()` holds valid saved `user_*` registers for the parent task and
    /// `parent.pml4_phys` is a valid page table root.
    pub unsafe fn fork_current_task() -> Result<usize, &'static str> {
        clone_current_task(0, 0, 0, 0, 0)
    }
}

pub mod spawn {
    use keira_mem::{pmm, vmm};

    use crate::scheduler::core::{
        CURRENT_TASK_IDX, MAX_TASKS, TASKS, TASK_STACK_PAGES, TASK_STACK_SIZE,
    };
    use crate::scheduler::lifecycle::reap_orphaned_zombies;
    use crate::types::{FileDescriptor, InterruptContext, Task, TaskState, MAX_FDS};

    #[cfg(test)]
    static mut TEST_TASK_STACKS: [[u8; 16384]; 16] = [[0; 16384]; 16];

    /// Creates and enqueues a new Ring 0 kernel thread starting at `entry_point`.
    ///
    /// # Safety
    ///
    /// Reads and mutates `TASKS` without acquiring `SCHEDULER_LOCK` (and calls
    /// `reap_orphaned_zombies()` when full, which locks `SCHEDULER_LOCK`). Callers must ensure
    /// `SCHEDULER_LOCK` is not held, `CURRENT_TASK_IDX < MAX_TASKS` and no concurrent CPU or
    /// interrupt handler races on `TASKS`, `CURRENT_TASK_IDX` or `TEST_TASK_STACKS`. PMM and VMM
    /// must be initialized outside `#[cfg(test)]` fallback and `entry_point` must be a valid
    /// kernel thread entry that does not return past its initial stack frame.
    pub unsafe fn spawn(name: &'static str, entry_point: fn()) -> Result<usize, &'static str> {
        let mut slot = None;
        for (i, item) in TASKS.iter().enumerate().take(MAX_TASKS) {
            if item.is_none() {
                slot = Some(i);
                break;
            }
        }

        if slot.is_none() {
            reap_orphaned_zombies();
            for (i, item) in TASKS.iter().enumerate().take(MAX_TASKS) {
                if item.is_none() {
                    slot = Some(i);
                    break;
                }
            }
        }

        let slot_idx = slot.ok_or("Scheduler: Maximum task limit reached")?;

        #[cfg(test)]
        let (stack_frame, stack_top) = match pmm::alloc_contiguous_frames(TASK_STACK_PAGES) {
            Some(f) => (f, f + TASK_STACK_SIZE),
            None => {
                let base = TEST_TASK_STACKS[slot_idx % 16].as_mut_ptr() as u64;
                (base, base + 16384)
            }
        };
        #[cfg(not(test))]
        let stack_frame = pmm::alloc_contiguous_frames(TASK_STACK_PAGES)
            .ok_or("Scheduler: Out of memory for task stack")?;
        #[cfg(not(test))]
        let stack_top = stack_frame + TASK_STACK_SIZE;

        let context_ptr =
            (stack_top - core::mem::size_of::<InterruptContext>() as u64) as *mut InterruptContext;

        #[cfg(target_arch = "x86_64")]
        {
            (*context_ptr).r15 = 0;
            (*context_ptr).r14 = 0;
            (*context_ptr).r13 = 0;
            (*context_ptr).r12 = 0;
            (*context_ptr).r11 = 0;
            (*context_ptr).r10 = 0;
            (*context_ptr).r9 = 0;
            (*context_ptr).r8 = 0;
            (*context_ptr).rdi = 0;
            (*context_ptr).rsi = 0;
            (*context_ptr).rbp = 0;
            (*context_ptr).rbx = 0;
            (*context_ptr).rdx = 0;
            (*context_ptr).rcx = 0;
            (*context_ptr).rax = 0;

            (*context_ptr).rip = entry_point as usize as u64;
            (*context_ptr).cs = 0x08;
            (*context_ptr).rflags = 0x202;
            (*context_ptr).rsp = stack_top;
            (*context_ptr).ss = 0x10;
        }

        #[cfg(target_arch = "x86")]
        {
            (*context_ptr).rdi = 0;
            (*context_ptr).rsi = 0;
            (*context_ptr).rbp = 0;
            (*context_ptr).esp_padding = 0;
            (*context_ptr).rbx = 0;
            (*context_ptr).rdx = 0;
            (*context_ptr).rcx = 0;
            (*context_ptr).rax = 0;

            (*context_ptr).rip = entry_point as usize as u32;
            (*context_ptr).cs = 0x08;
            (*context_ptr).rflags = 0x202;
            (*context_ptr).rsp = stack_top as u32;
            (*context_ptr).ss = 0x10;
        }

        let mut child_cwd = [0u8; 128];
        child_cwd[0] = b'/';
        let mut parent_cwd_len = 1usize;
        let mut parent_pml4 = vmm::active_pml4();
        let parent_id = CURRENT_TASK_IDX;
        let mut parent_pgid = slot_idx;
        let mut parent_sid = slot_idx;
        if let Some(ref parent) = TASKS[parent_id] {
            child_cwd[..parent.cwd_len].copy_from_slice(&parent.cwd[..parent.cwd_len]);
            parent_cwd_len = parent.cwd_len;
            parent_pml4 = parent.pml4_phys;
            parent_pgid = parent.pgid;
            parent_sid = parent.sid;
        }
        let new_task = Task {
            id: slot_idx,
            name,
            comm: [0u8; 32],
            comm_len: 0,
            cmdline: [0u8; 128],
            cmdline_len: 0,
            rsp: context_ptr as u64,
            stack_addr: stack_frame,
            state: TaskState::Ready,
            fds: [FileDescriptor::new(); MAX_FDS],
            program_break: 0,
            program_break_start: 0,
            cwd: child_cwd,
            cwd_len: parent_cwd_len,
            parent_id,
            pml4_phys: parent_pml4,
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
            tgid: slot_idx,
            is_thread: false,
            clear_child_tid: 0,
            tls: 0,
            cpu_ticks: 0,
            switches: 0,
            stack_aslr_offset: 0,
            mmap_aslr_offset: 0,
            brk_aslr_offset: 0,
            rlimits: crate::rlimit::default_rlimits(),
            pgid: parent_pgid,
            sid: parent_sid,
        };

        TASKS[slot_idx] = Some(new_task);
        let core_id = keira_arch::cpu::get_current_core_id();
        crate::scheduler::work_stealing::push_local(core_id, slot_idx);

        Ok(slot_idx)
    }

    /// Creates and enqueues a new Ring 3 user task starting at `entry_point` and `user_rsp`.
    ///
    /// # Safety
    ///
    /// Reads and mutates `TASKS` without acquiring `SCHEDULER_LOCK` (and calls
    /// `reap_orphaned_zombies()` when full, which locks `SCHEDULER_LOCK`). Callers must ensure
    /// `SCHEDULER_LOCK` is not held, `CURRENT_TASK_IDX < MAX_TASKS` and no concurrent CPU or
    /// interrupt handler races on `TASKS`, `CURRENT_TASK_IDX` or `TEST_TASK_STACKS`.
    /// `pml4_phys` must be a valid physical page table root with kernel mappings intact, while
    /// `entry_point` and `user_rsp` must reference valid Ring 3 code and an initialized user
    /// stack in that address space.
    pub unsafe fn spawn_user(
        name: &'static str,
        entry_point: u64,
        user_rsp: u64,
        pml4_phys: u64,
    ) -> Result<usize, &'static str> {
        let mut slot = None;
        for (i, item) in TASKS.iter().enumerate().take(MAX_TASKS) {
            if item.is_none() {
                slot = Some(i);
                break;
            }
        }

        if slot.is_none() {
            reap_orphaned_zombies();
            for (i, item) in TASKS.iter().enumerate().take(MAX_TASKS) {
                if item.is_none() {
                    slot = Some(i);
                    break;
                }
            }
        }

        let slot_idx = slot.ok_or("Scheduler: Maximum task limit reached")?;

        #[cfg(test)]
        let (stack_frame, stack_top) = match pmm::alloc_contiguous_frames(TASK_STACK_PAGES) {
            Some(f) => (f, f + TASK_STACK_SIZE),
            None => {
                let base = TEST_TASK_STACKS[slot_idx % 16].as_mut_ptr() as u64;
                (base, base + 16384)
            }
        };
        #[cfg(not(test))]
        let stack_frame = pmm::alloc_contiguous_frames(TASK_STACK_PAGES)
            .ok_or("Scheduler: Out of memory for task stack")?;
        #[cfg(not(test))]
        let stack_top = stack_frame + TASK_STACK_SIZE;

        let context_ptr =
            (stack_top - core::mem::size_of::<InterruptContext>() as u64) as *mut InterruptContext;

        #[cfg(target_arch = "x86_64")]
        {
            (*context_ptr).r15 = 0;
            (*context_ptr).r14 = 0;
            (*context_ptr).r13 = 0;
            (*context_ptr).r12 = 0;
            (*context_ptr).r11 = 0;
            (*context_ptr).r10 = 0;
            (*context_ptr).r9 = 0;
            (*context_ptr).r8 = 0;
            (*context_ptr).rdi = 0;
            (*context_ptr).rsi = 0;
            (*context_ptr).rbp = 0;
            (*context_ptr).rbx = 0;
            (*context_ptr).rdx = 0;
            (*context_ptr).rcx = 0;
            (*context_ptr).rax = 0;

            (*context_ptr).rip = entry_point;
            (*context_ptr).cs = 0x2B;
            (*context_ptr).rflags = 0x202;
            (*context_ptr).rsp = user_rsp;
            (*context_ptr).ss = 0x23;
        }

        #[cfg(target_arch = "x86")]
        {
            (*context_ptr).rdi = 0;
            (*context_ptr).rsi = 0;
            (*context_ptr).rbp = 0;
            (*context_ptr).esp_padding = 0;
            (*context_ptr).rbx = 0;
            (*context_ptr).rdx = 0;
            (*context_ptr).rcx = 0;
            (*context_ptr).rax = 0;

            (*context_ptr).rip = entry_point as u32;
            (*context_ptr).cs = 0x1B;
            (*context_ptr).rflags = 0x202;
            (*context_ptr).rsp = user_rsp as u32;
            (*context_ptr).ss = 0x23;
        }

        let mut child_cwd = [0u8; 128];
        child_cwd[0] = b'/';
        let mut parent_cwd_len = 1usize;
        let mut child_fds = [FileDescriptor::new(); MAX_FDS];
        let parent_id = CURRENT_TASK_IDX;
        let mut parent_uid = 0u32;
        let mut parent_gid = 0u32;
        let mut parent_euid = 0u32;
        let mut parent_egid = 0u32;
        let mut parent_rlimits = crate::rlimit::default_rlimits();
        let mut parent_pgid = slot_idx;
        let mut parent_sid = slot_idx;

        if let Some(ref parent) = TASKS[parent_id] {
            child_cwd[..parent.cwd_len].copy_from_slice(&parent.cwd[..parent.cwd_len]);
            parent_cwd_len = parent.cwd_len;
            child_fds = parent.fds;
            parent_uid = parent.uid;
            parent_gid = parent.gid;
            parent_euid = parent.euid;
            parent_egid = parent.egid;
            parent_rlimits = parent.rlimits;
            parent_pgid = parent.pgid;
            parent_sid = parent.sid;
        }

        for (fd, item) in child_fds.iter_mut().enumerate().take(3) {
            if !item.is_open {
                item.is_open = true;
                item.write_mode = fd != 0;
                item.path[..8].copy_from_slice(b"/dev/tty");
                item.path_len = 8;
            }
        }

        #[cfg(target_arch = "x86_64")]
        let default_brk = 0x600000000000;
        #[cfg(target_arch = "x86")]
        let default_brk = 0x0200_0000;

        let (stack_aslr, mmap_aslr, brk_aslr) = crate::aslr::generate_task_aslr_offsets();

        let new_task = Task {
            id: slot_idx,
            name,
            comm: [0u8; 32],
            comm_len: 0,
            cmdline: [0u8; 128],
            cmdline_len: 0,
            rsp: context_ptr as u64,
            stack_addr: stack_frame,
            state: TaskState::Ready,
            fds: child_fds,
            program_break: default_brk,
            program_break_start: default_brk,
            cwd: child_cwd,
            cwd_len: parent_cwd_len,
            parent_id,
            pml4_phys,
            exit_code: 0,
            is_user: true,
            uid: parent_uid,
            gid: parent_gid,
            euid: parent_euid,
            egid: parent_egid,
            saved_sigcontext: None,
            signal_mask: 0,
            pending_signals: 0,
            is_orphan: false,
            tgid: slot_idx,
            is_thread: false,
            clear_child_tid: 0,
            tls: 0,
            cpu_ticks: 0,
            switches: 0,
            stack_aslr_offset: stack_aslr,
            mmap_aslr_offset: mmap_aslr,
            brk_aslr_offset: brk_aslr,
            rlimits: parent_rlimits,
            pgid: parent_pgid,
            sid: parent_sid,
        };

        TASKS[slot_idx] = Some(new_task);
        let core_id = keira_arch::cpu::get_current_core_id();
        crate::scheduler::work_stealing::push_local(core_id, slot_idx);

        Ok(slot_idx)
    }
}

pub use self::clone::*;
pub use self::fork::*;
pub use self::spawn::*;
