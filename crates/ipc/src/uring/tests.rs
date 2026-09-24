// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for asynchronous `io_uring` engine.

#[cfg(test)]
mod test {
    use core::sync::atomic::Ordering;

    use crate::uring::*;

    #[test]
    fn test_io_uring_setup_and_fsync_op() {
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
            opcode: 255,
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
        assert_eq!(cqe.res, -95);
    }
}
