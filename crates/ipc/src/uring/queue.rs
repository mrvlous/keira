// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! High-Performance Bare-Metal Asynchronous I/O Engine (`io_uring`).
//!
//! Implements Linux ABI-compatible Submission Queue (SQ) and Completion Queue (CQ)
//! lockless ring buffers with atomic head and tail pointers, asynchronous opcode dispatch,
//! and memory-mapped parameter sharing between userland and kernel.

#![allow(static_mut_refs)]

use core::sync::atomic::{AtomicU32, Ordering};

// Standard Linux io_uring Opcodes.
pub const IORING_OP_NOP: u8 = 0;
pub const IORING_OP_READV: u8 = 1;
pub const IORING_OP_WRITEV: u8 = 2;
pub const IORING_OP_FSYNC: u8 = 3;
pub const IORING_OP_READ_FIXED: u8 = 4;
pub const IORING_OP_WRITE_FIXED: u8 = 5;
pub const IORING_OP_POLL_ADD: u8 = 6;
pub const IORING_OP_POLL_REMOVE: u8 = 7;
pub const IORING_OP_SYNC_FILE_RANGE: u8 = 8;
pub const IORING_OP_SENDMSG: u8 = 9;
pub const IORING_OP_RECVMSG: u8 = 10;
pub const IORING_OP_TIMEOUT: u8 = 11;
pub const IORING_OP_CLOSE: u8 = 19;
pub const IORING_OP_STATX: u8 = 21;
pub const IORING_OP_READ: u8 = 22;
pub const IORING_OP_WRITE: u8 = 23;

// SQE submission flags.
pub const IOSQE_FIXED_FILE: u8 = 1 << 0;
pub const IOSQE_IO_DRAIN: u8 = 1 << 1;
pub const IOSQE_IO_LINK: u8 = 1 << 2;
pub const IOSQE_IO_HARDLINK: u8 = 1 << 3;
pub const IOSQE_ASYNC: u8 = 1 << 4;
pub const IOSQE_BUFFER_SELECT: u8 = 1 << 5;

// Setup flags for io_uring_setup.
pub const IORING_SETUP_IOPOLL: u32 = 1 << 0;
pub const IORING_SETUP_SQPOLL: u32 = 1 << 1;
pub const IORING_SETUP_SQ_AFF: u32 = 1 << 2;
pub const IORING_SETUP_CQSIZE: u32 = 1 << 3;
pub const IORING_SETUP_CLAMP: u32 = 1 << 4;
pub const IORING_SETUP_ATTACH_WQ: u32 = 1 << 5;

// Enter flags for io_uring_enter.
pub const IORING_ENTER_GETEVENTS: u32 = 1 << 0;
pub const IORING_ENTER_SQ_WAKEUP: u32 = 1 << 1;
pub const IORING_ENTER_SQ_WAIT: u32 = 1 << 2;

// Kernel-supported io_uring feature flags.
pub const IORING_FEAT_SINGLE_MMAP: u32 = 1 << 0;
pub const IORING_FEAT_NODROP: u32 = 1 << 1;
pub const IORING_FEAT_SUBMIT_STABLE: u32 = 1 << 2;

// System ring limits.
pub const MAX_IO_URING_INSTANCES: usize = 8;
pub const MAX_RING_ENTRIES: usize = 64;

/// Linux ABI-compatible Submission Queue Entry (SQE).
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SubmissionQueueEntry {
    pub opcode: u8,
    pub flags: u8,
    pub ioprio: u16,
    pub fd: i32,
    pub off: u64,
    pub addr: u64,
    pub len: u32,
    pub rw_flags: u32,
    pub user_data: u64,
    pub buf_index: u16,
    pub personality: u16,
    pub splice_fd_in: i32,
    pub pad2: [u64; 2],
}

impl Default for SubmissionQueueEntry {
    fn default() -> Self {
        Self {
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
        }
    }
}

/// Linux ABI-compatible Completion Queue Entry (CQE).
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct CompletionQueueEntry {
    pub user_data: u64,
    pub res: i32,
    pub flags: u32,
}

/// Offsets structure for Submission Queue memory layout.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct IoSqringOffsets {
    pub head: u32,
    pub tail: u32,
    pub ring_mask: u32,
    pub ring_entries: u32,
    pub flags: u32,
    pub dropped: u32,
    pub array: u32,
    pub resv1: u32,
    pub user_addr: u64,
}

/// Offsets structure for Completion Queue memory layout.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct IoCqringOffsets {
    pub head: u32,
    pub tail: u32,
    pub ring_mask: u32,
    pub ring_entries: u32,
    pub overflow: u32,
    pub cqes: u32,
    pub flags: u32,
    pub resv1: u32,
    pub user_addr: u64,
}

/// Parameters passed to and returned from `io_uring_setup`.
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct IoUringParams {
    pub sq_entries: u32,
    pub cq_entries: u32,
    pub flags: u32,
    pub sq_thread_cpu: u32,
    pub sq_thread_idle: u32,
    pub features: u32,
    pub wq_fd: u32,
    pub resv: [u32; 3],
    pub sq_off: IoSqringOffsets,
    pub cq_off: IoCqringOffsets,
}

/// A single live instance of an `io_uring` ring buffer channel.
pub struct IoUringRing {
    pub id: u32,
    pub in_use: bool,
    pub flags: u32,
    pub sq_entries: u32,
    pub cq_entries: u32,
    pub sq_mask: u32,
    pub cq_mask: u32,

    pub sq_head: AtomicU32,
    pub sq_tail: AtomicU32,
    pub cq_head: AtomicU32,
    pub cq_tail: AtomicU32,

    pub sqes: [SubmissionQueueEntry; MAX_RING_ENTRIES],
    pub cqes: [CompletionQueueEntry; MAX_RING_ENTRIES],

    pub submitted_count: AtomicU32,
    pub completed_count: AtomicU32,
    pub overflow_count: AtomicU32,
}

pub type IoUringInstance = IoUringRing;

impl IoUringRing {
    pub const fn new(id: u32) -> Self {
        Self {
            id,
            in_use: false,
            flags: 0,
            sq_entries: 0,
            cq_entries: 0,
            sq_mask: 0,
            cq_mask: 0,
            sq_head: AtomicU32::new(0),
            sq_tail: AtomicU32::new(0),
            cq_head: AtomicU32::new(0),
            cq_tail: AtomicU32::new(0),
            sqes: [SubmissionQueueEntry {
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
            }; MAX_RING_ENTRIES],
            cqes: [CompletionQueueEntry {
                user_data: 0,
                res: 0,
                flags: 0,
            }; MAX_RING_ENTRIES],
            submitted_count: AtomicU32::new(0),
            completed_count: AtomicU32::new(0),
            overflow_count: AtomicU32::new(0),
        }
    }

    /// Reset ring state for new setup.
    pub fn reset(&mut self, entries: u32, flags: u32) {
        let actual_sq = entries
            .clamp(1, MAX_RING_ENTRIES as u32)
            .next_power_of_two();
        let actual_cq = (actual_sq * 2).clamp(2, MAX_RING_ENTRIES as u32);

        self.in_use = true;
        self.flags = flags;
        self.sq_entries = actual_sq;
        self.cq_entries = actual_cq;
        self.sq_mask = actual_sq - 1;
        self.cq_mask = actual_cq - 1;

        self.sq_head.store(0, Ordering::Relaxed);
        self.sq_tail.store(0, Ordering::Relaxed);
        self.cq_head.store(0, Ordering::Relaxed);
        self.cq_tail.store(0, Ordering::Relaxed);

        self.submitted_count.store(0, Ordering::Relaxed);
        self.completed_count.store(0, Ordering::Relaxed);
        self.overflow_count.store(0, Ordering::Relaxed);

        for sqe in self.sqes.iter_mut() {
            *sqe = SubmissionQueueEntry::default();
        }
        for cqe in self.cqes.iter_mut() {
            *cqe = CompletionQueueEntry::default();
        }
    }
}

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
        // Find free ring slot
        let mut chosen_slot = None;
        for (idx, r) in RINGS.iter_mut().enumerate() {
            if !r.in_use {
                chosen_slot = Some(idx);
                break;
            }
        }

        let slot_idx = match chosen_slot {
            Some(idx) => idx,
            None => {
                // Reuse slot 0 if all occupied
                0
            }
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
            (*params).sq_off.user_addr = core::ptr::addr_of!(ring.sqes) as u64;

            (*params).cq_off.head = 0;
            (*params).cq_off.tail = 4;
            (*params).cq_off.ring_mask = 8;
            (*params).cq_off.ring_entries = 12;
            (*params).cq_off.overflow = 16;
            (*params).cq_off.cqes = 20;
            (*params).cq_off.flags = 24;
            (*params).cq_off.user_addr = core::ptr::addr_of!(ring.cqes) as u64;
        }

        Ok(slot_idx as u64)
    }
}

/// Backward-compatible setup_ring call (creates ring 0).
pub fn setup_ring(entries: u32) -> Result<u64, &'static str> {
    setup_ring_ext(entries, 0)
}

/// Process a single submission queue entry.
fn process_single_sqe(sqe: &SubmissionQueueEntry) -> CompletionQueueEntry {
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
                // Stdin console / serial read
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

            // Read from task file descriptor or direct memory
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

            // Fallback for direct memory/pipe operations
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
                // Stdout / Stderr console output
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
            // Unsupported opcode
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

        // Calculate pending count: if to_submit > 0, cap to min(to_submit, pending or entries)
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

            // Push to CQ ring
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
pub fn get_ring(ring_id: u32) -> Option<&'static IoUringRing> {
    let slot = ring_id as usize;
    if slot < MAX_IO_URING_INSTANCES {
        unsafe {
            if RINGS[slot].in_use {
                return Some(&RINGS[slot]);
            }
        }
    }
    None
}

/// Get mutable reference to active ring by ID.
pub fn get_ring_mut(ring_id: u32) -> Option<&'static mut IoUringRing> {
    let slot = ring_id as usize;
    if slot < MAX_IO_URING_INSTANCES {
        unsafe {
            if RINGS[slot].in_use {
                return Some(&mut RINGS[slot]);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_uring_setup_ring_and_offsets() {
        let mut params = IoUringParams::default();
        let ring_id = setup_ring_ext(16, core::ptr::addr_of_mut!(params) as u64)
            .expect("setup_ring_ext failed");
        assert!(ring_id < MAX_IO_URING_INSTANCES as u64);

        assert_eq!(params.sq_entries, 16);
        assert_eq!(params.cq_entries, 32);
        assert_eq!(
            params.features,
            IORING_FEAT_SINGLE_MMAP | IORING_FEAT_NODROP | IORING_FEAT_SUBMIT_STABLE
        );
        assert_ne!(params.sq_off.user_addr, 0);
        assert_ne!(params.cq_off.user_addr, 0);

        let ring = get_ring(ring_id as u32).expect("ring not found");
        assert!(ring.in_use);
        assert_eq!(ring.sq_entries, 16);
        assert_eq!(ring.sq_mask, 15);
        assert_eq!(ring.cq_entries, 32);
        assert_eq!(ring.cq_mask, 31);
    }

    #[test]
    fn test_io_uring_nop_lifecycle() {
        let ring_id = setup_ring(8).expect("setup_ring failed") as u32;
        let ring = get_ring_mut(ring_id).expect("ring not found");

        let sqe_idx = (ring.sq_tail.load(Ordering::Relaxed) & ring.sq_mask) as usize;
        ring.sqes[sqe_idx] = SubmissionQueueEntry {
            opcode: IORING_OP_NOP,
            flags: 0,
            ioprio: 0,
            fd: -1,
            off: 0,
            addr: 0,
            len: 0,
            rw_flags: 0,
            user_data: 0xDEADBEEF_CAFE0001,
            buf_index: 0,
            personality: 0,
            splice_fd_in: -1,
            pad2: [0; 2],
        };
        ring.sq_tail.fetch_add(1, Ordering::Relaxed);

        let count = enter_ring_ext(ring_id, 1, 1, 0).expect("enter_ring failed");
        assert_eq!(count, 1);

        let cq_head = ring.cq_head.load(Ordering::Relaxed);
        let cqe_idx = (cq_head & ring.cq_mask) as usize;
        let cqe = ring.cqes[cqe_idx];

        assert_eq!(cqe.user_data, 0xDEADBEEF_CAFE0001);
        assert_eq!(cqe.res, 0);
        ring.cq_head.fetch_add(1, Ordering::Relaxed);
    }

    #[test]
    fn test_io_uring_fsync_op() {
        let ring_id = setup_ring(4).expect("setup failed") as u32;
        let ring = get_ring_mut(ring_id).expect("ring not found");

        let sqe_idx = (ring.sq_tail.load(Ordering::Relaxed) & ring.sq_mask) as usize;
        ring.sqes[sqe_idx] = SubmissionQueueEntry {
            opcode: IORING_OP_FSYNC,
            flags: 0,
            ioprio: 0,
            fd: 3,
            off: 0,
            addr: 0,
            len: 0,
            rw_flags: 0,
            user_data: 0x7777,
            buf_index: 0,
            personality: 0,
            splice_fd_in: -1,
            pad2: [0; 2],
        };
        ring.sq_tail.fetch_add(1, Ordering::Relaxed);

        let res = enter_ring_ext(ring_id, 1, 1, 0).expect("enter failed");
        assert_eq!(res, 1);

        let cqe = ring.cqes[(ring.cq_head.load(Ordering::Relaxed) & ring.cq_mask) as usize];
        assert_eq!(cqe.user_data, 0x7777);
        assert_eq!(cqe.res, 0);
    }

    #[test]
    fn test_io_uring_unsupported_opcode_handling() {
        let ring_id = setup_ring(4).expect("setup failed") as u32;
        let ring = get_ring_mut(ring_id).expect("ring not found");

        let sqe_idx = (ring.sq_tail.load(Ordering::Relaxed) & ring.sq_mask) as usize;
        ring.sqes[sqe_idx] = SubmissionQueueEntry {
            opcode: 255, // Unknown
            flags: 0,
            ioprio: 0,
            fd: 0,
            off: 0,
            addr: 0,
            len: 0,
            rw_flags: 0,
            user_data: 0x9999,
            buf_index: 0,
            personality: 0,
            splice_fd_in: -1,
            pad2: [0; 2],
        };
        ring.sq_tail.fetch_add(1, Ordering::Relaxed);

        let res = enter_ring_ext(ring_id, 1, 1, 0).expect("enter failed");
        assert_eq!(res, 1);

        let cqe = ring.cqes[(ring.cq_head.load(Ordering::Relaxed) & ring.cq_mask) as usize];
        assert_eq!(cqe.user_data, 0x9999);
        assert_eq!(cqe.res, -95); // -EOPNOTSUPP
    }
}
