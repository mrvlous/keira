// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! io_uring asynchronous engine runtime and request processor.

#![allow(static_mut_refs)]

use core::sync::atomic::Ordering;

use crate::uring::cq::entry::CompletionQueueEntry;
use crate::uring::engine::types::*;
use crate::uring::sq::entry::SubmissionQueueEntry;

// Global kernel table of io_uring rings
pub static mut RINGS: [IoUringRing; MAX_IO_URING_INSTANCES] = [
    IoUringRing::new(0),
    IoUringRing::new(1),
    IoUringRing::new(2),
    IoUringRing::new(3),
    IoUringRing::new(4),
    IoUringRing::new(5),
    IoUringRing::new(6),
    IoUringRing::new(7),
];

// Flat backward-compatible references for default ring 0
pub static mut SQ_ENTRIES: [SubmissionQueueEntry; 32] = [SubmissionQueueEntry {
    opcode: 0,
    flags: 0,
    ioprio: 0,
    fd: -1,
    off: 0,
    addr: 0,
    len: 0,
    rw_flags: 0,
    user_data: 0,
    buf_index: 0,
    personality: 0,
    splice_fd_in: -1,
    pad2: [0; 2],
}; 32];

pub static mut CQ_ENTRIES: [CompletionQueueEntry; 32] = [CompletionQueueEntry {
    user_data: 0,
    res: 0,
    flags: 0,
}; 32];

/// Setup io_uring submission & completion ring buffers with optional parameter pointer.
pub fn setup_ring_ext(entries: u32, params_ptr: u64) -> Result<u64, &'static str> {
    if entries == 0 || entries > (MAX_RING_ENTRIES as u32 * 2) {
        return Err("Invalid entry count");
    }

    unsafe {
        let mut chosen_slot = None;
        for (idx, r) in RINGS.iter_mut().enumerate() {
            if !r.in_use {
                chosen_slot = Some(idx);
                break;
            }
        }

        let slot_idx = match chosen_slot {
            Some(idx) => idx,
            None => 0,
        };

        let ring = &mut RINGS[slot_idx];
        ring.reset(entries, 0);

        if params_ptr != 0 {
            let params = params_ptr as *mut IoUringParams;
            (*params).sq_entries = ring.sq_entries;
            (*params).cq_entries = ring.cq_entries;
            (*params).flags = ring.flags;
            (*params).features =
                IORING_FEAT_SINGLE_MMAP | IORING_FEAT_NODROP | IORING_FEAT_SUBMIT_STABLE;
            (*params).sq_thread_cpu = 0;
            (*params).sq_thread_idle = 0;
            (*params).wq_fd = 0;

            (*params).sq_off.head = 0;
            (*params).sq_off.tail = 4;
            (*params).sq_off.ring_mask = 8;
            (*params).sq_off.ring_entries = 12;
            (*params).sq_off.flags = 16;
            (*params).sq_off.dropped = 20;
            (*params).sq_off.array = 24;
            (*params).sq_off.resv1 = 0;
            (*params).sq_off.user_addr = core::ptr::addr_of!(ring.sqes) as u64;
            (*params).sq_off.pad = [0; 2];

            (*params).cq_off.head = 0;
            (*params).cq_off.tail = 4;
            (*params).cq_off.ring_mask = 8;
            (*params).cq_off.ring_entries = 12;
            (*params).cq_off.overflow = 16;
            (*params).cq_off.cqes = 20;
            (*params).cq_off.flags = 24;
            (*params).cq_off.resv1 = 0;
            (*params).cq_off.user_addr = core::ptr::addr_of!(ring.cqes) as u64;
            (*params).cq_off.pad = [0; 2];

            #[cfg(target_os = "none")]
            {
                let page_size = keira_mem::pmm::PAGE_SIZE as usize;
                let ring_start = (core::ptr::addr_of!(RINGS) as usize) & !(page_size - 1);
                let ring_end = (core::ptr::addr_of!(RINGS) as usize
                    + core::mem::size_of_val(&RINGS)
                    + page_size
                    - 1)
                    & !(page_size - 1);
                for page in (ring_start..ring_end).step_by(page_size) {
                    let _ = keira_mem::vmm::map_page(
                        page as u64,
                        page as u64,
                        keira_mem::vmm::PAGE_USER
                            | keira_mem::vmm::PAGE_WRITABLE
                            | keira_mem::vmm::PAGE_PRESENT,
                    );
                    keira_arch::cpu::invlpg(page);
                }
            }
        }

        Ok(slot_idx as u64)
    }
}

/// Backward-compatible setup_ring call (creates ring 0).
pub fn setup_ring(entries: u32) -> Result<u64, &'static str> {
    setup_ring_ext(entries, 0)
}

/// Process a single submission queue entry.
pub fn process_single_sqe(sqe: &SubmissionQueueEntry) -> CompletionQueueEntry {
    let mut cqe = CompletionQueueEntry {
        user_data: sqe.user_data,
        res: 0,
        flags: 0,
    };

    match sqe.opcode {
        IORING_OP_NOP => {
            cqe.res = 0;
        }
        IORING_OP_READ => {
            let fd = sqe.fd;
            let buf_ptr = sqe.addr as *mut u8;
            let len = sqe.len as usize;
            let off = sqe.off;

            if buf_ptr.is_null() && len > 0 {
                cqe.res = -14; // -EFAULT
                return cqe;
            }

            if fd < 0 {
                cqe.res = -9; // -EBADF
                return cqe;
            }

            if len == 0 {
                cqe.res = 0;
                return cqe;
            }

            if fd == 0 {
                let dest = unsafe { core::slice::from_raw_parts_mut(buf_ptr, len) };
                let mut count = 0;
                while count < len {
                    let maybe_ch = unsafe { keira_io::ps2::pop_input_char() };
                    if let Some(ch) = maybe_ch {
                        dest[count] = ch;
                        count += 1;
                    } else if keira_io::serial::has_byte() {
                        let ch = keira_io::serial::read_byte();
                        if ch != 0 {
                            dest[count] = ch;
                            count += 1;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                cqe.res = count as i32;
                return cqe;
            }

            unsafe {
                let task_idx = keira_task::scheduler::CURRENT_TASK_IDX;
                if let Some(ref mut task) = keira_task::scheduler::TASKS[task_idx] {
                    if (fd as usize) < task.fds.len() && task.fds[fd as usize].is_open {
                        let f_desc = &mut task.fds[fd as usize];
                        if f_desc.is_pipe && !f_desc.pipe_write {
                            let dest = core::slice::from_raw_parts_mut(buf_ptr, len);
                            let bytes = crate::pipe::read_pipe(dest);
                            cqe.res = bytes as i32;
                            return cqe;
                        }

                        let path_len = (f_desc.path_len as usize).min(64);
                        if let Ok(path_str) = core::str::from_utf8(&f_desc.path[..path_len]) {
                            let dest = core::slice::from_raw_parts_mut(buf_ptr, len);
                            let actual_offset = if off != 0 { off } else { f_desc.offset };
                            match keira_fs::vfs::read_file_offset(path_str, actual_offset, dest) {
                                Ok(bytes_read) => {
                                    if off == 0 {
                                        f_desc.offset += bytes_read as u64;
                                    }
                                    cqe.res = bytes_read as i32;
                                }
                                Err(_) => {
                                    cqe.res = -5; // -EIO
                                }
                            }
                            return cqe;
                        }
                    }
                }
            }

            cqe.res = -9; // -EBADF
        }
        IORING_OP_WRITE => {
            let fd = sqe.fd;
            let buf_ptr = sqe.addr as *const u8;
            let len = sqe.len as usize;
            let off = sqe.off;

            if buf_ptr.is_null() && len > 0 {
                cqe.res = -14; // -EFAULT
                return cqe;
            }

            if fd < 0 {
                cqe.res = -9; // -EBADF
                return cqe;
            }

            if len == 0 {
                cqe.res = 0;
                return cqe;
            }

            if fd == 1 || fd == 2 {
                let src = unsafe { core::slice::from_raw_parts(buf_ptr, len) };
                if let Ok(s) = core::str::from_utf8(src) {
                    keira_io::vga::print_str(s);
                } else {
                    for &b in src {
                        keira_io::vga::putchar(b);
                    }
                }
                cqe.res = len as i32;
                return cqe;
            }

            unsafe {
                let task_idx = keira_task::scheduler::CURRENT_TASK_IDX;
                if let Some(ref mut task) = keira_task::scheduler::TASKS[task_idx] {
                    if (fd as usize) < task.fds.len() && task.fds[fd as usize].is_open {
                        let f_desc = &mut task.fds[fd as usize];
                        if f_desc.is_pipe && f_desc.pipe_write {
                            let src = core::slice::from_raw_parts(buf_ptr, len);
                            let bytes = crate::pipe::write_pipe(src);
                            cqe.res = bytes as i32;
                            return cqe;
                        }

                        let path_len = (f_desc.path_len as usize).min(64);
                        if let Ok(path_str) = core::str::from_utf8(&f_desc.path[..path_len]) {
                            let src = core::slice::from_raw_parts(buf_ptr, len);
                            let actual_offset = if off != 0 { off } else { f_desc.offset };
                            match keira_fs::vfs::write_file_offset(path_str, actual_offset, src) {
                                Ok(bytes_written) => {
                                    if off == 0 {
                                        f_desc.offset += bytes_written as u64;
                                    }
                                    cqe.res = bytes_written as i32;
                                }
                                Err(_) => {
                                    cqe.res = -5; // -EIO
                                }
                            }
                            return cqe;
                        }
                    }
                }
            }

            cqe.res = -9; // -EBADF
        }
        IORING_OP_FSYNC => {
            unsafe {
                let _ = keira_fs::flush_dirty_sectors();
            }
            cqe.res = 0;
        }
        IORING_OP_CLOSE => {
            let fd = sqe.fd;
            unsafe {
                let task_idx = keira_task::scheduler::CURRENT_TASK_IDX;
                if let Some(ref mut task) = keira_task::scheduler::TASKS[task_idx] {
                    if (fd as usize) < task.fds.len() && task.fds[fd as usize].is_open {
                        task.fds[fd as usize].is_open = false;
                        cqe.res = 0;
                        return cqe;
                    }
                }
            }
            cqe.res = -9; // -EBADF
        }
        _ => {
            cqe.res = -95; // -EOPNOTSUPP
        }
    }

    cqe
}

/// Submit queued async I/O requests and reap completed results on specified ring.
pub fn enter_ring_ext(
    ring_id: u32,
    to_submit: u32,
    _min_complete: u32,
    _flags: u32,
) -> Result<u64, &'static str> {
    let slot = ring_id as usize;
    if slot >= MAX_IO_URING_INSTANCES {
        return Err("Invalid ring descriptor");
    }

    unsafe {
        let ring = &mut RINGS[slot];
        if !ring.in_use {
            return Err("Ring not initialized");
        }

        let sq_head = ring.sq_head.load(Ordering::Acquire);
        let sq_tail = ring.sq_tail.load(Ordering::Acquire);

        let available = if sq_tail >= sq_head {
            sq_tail - sq_head
        } else {
            (sq_tail + ring.sq_entries) - sq_head
        };

        let submit_count = if to_submit > 0 {
            to_submit.min(ring.sq_entries)
        } else {
            available.min(ring.sq_entries)
        };

        let mut processed = 0u32;
        let mut curr_head = sq_head;

        while processed < submit_count {
            let sqe_idx = (curr_head & ring.sq_mask) as usize;
            let sqe = ring.sqes[sqe_idx];

            let cqe = process_single_sqe(&sqe);

            let cq_tail = ring.cq_tail.load(Ordering::Acquire);
            let cqe_idx = (cq_tail & ring.cq_mask) as usize;
            ring.cqes[cqe_idx] = cqe;

            ring.cq_tail.fetch_add(1, Ordering::Release);
            curr_head = curr_head.wrapping_add(1);
            processed += 1;
        }

        ring.sq_head.store(curr_head, Ordering::Release);
        ring.submitted_count.fetch_add(processed, Ordering::Relaxed);
        ring.completed_count.fetch_add(processed, Ordering::Relaxed);

        Ok(processed as u64)
    }
}

/// Backward-compatible enter_ring call (operates on ring 0).
pub fn enter_ring(to_submit: u32, min_complete: u32) -> Result<u32, &'static str> {
    match enter_ring_ext(0, to_submit, min_complete, 0) {
        Ok(count) => Ok(count as u32),
        Err(e) => Err(e),
    }
}

/// Get reference to active ring by ID.
pub fn get_ring(id: u32) -> Option<&'static IoUringRing> {
    if (id as usize) < MAX_IO_URING_INSTANCES {
        unsafe {
            if RINGS[id as usize].in_use {
                return Some(&RINGS[id as usize]);
            }
        }
    }
    None
}

/// Get mutable reference to active ring by ID.
pub fn get_ring_mut(id: u32) -> Option<&'static mut IoUringRing> {
    if (id as usize) < MAX_IO_URING_INSTANCES {
        unsafe {
            if RINGS[id as usize].in_use {
                return Some(&mut RINGS[id as usize]);
            }
        }
    }
    None
}
