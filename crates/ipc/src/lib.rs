// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![no_std]

//! Inter-Process Communication (Pipes, Futex, Epoll, EventFD, MQueue, Splice, POSIX SHM, io_uring).

pub mod event;
pub mod futex;
pub mod mqueue;
pub mod pipe;
pub mod shm;
pub mod uring;

pub use event::epoll::{
    self, sys_epoll_create, sys_epoll_ctl, EpollInstance, EPOLL_CTL_ADD, EPOLL_CTL_DEL,
    EPOLL_CTL_MOD,
};
pub use event::eventfd::{self, sys_eventfd, sys_signalfd, EventFd};
pub use futex::sync::{
    self as futex_sync, sys_futex, FUTEX_CMP_REQUEUE, FUTEX_CMP_REQUEUE_PI, FUTEX_FD,
    FUTEX_LOCK_PI, FUTEX_REQUEUE, FUTEX_TRYLOCK_PI, FUTEX_UNLOCK_PI, FUTEX_WAIT, FUTEX_WAIT_BITSET,
    FUTEX_WAIT_REQUEUE_PI, FUTEX_WAKE, FUTEX_WAKE_BITSET, FUTEX_WAKE_OP,
};
pub use mqueue::queue::{
    self as mqueue_queue, get_mqueue_stats, get_mqueue_table, mq_open, mq_receive, mq_send,
    mq_unlink, mq_unlink_by_id, sys_mq_open, MQueueMessage, PosixMessageQueue, MAX_MQUEUES,
    MQUEUE_MAX_MSGS, MQUEUE_MSG_SIZE,
};
pub use pipe::fifo::{
    create_pipe, read_pipe, write_pipe, PipeBuffer, PIPE_BUFFER_SIZE, SYSTEM_PIPE,
};
pub use pipe::splice::{sys_splice, sys_vmsplice};
pub use shm::segment::{
    create_sem, create_shm, get_sem_table, get_shm_frame, get_shm_table, remove_sem, remove_shm,
    sys_shm_sem, Semaphore, ShmSegment, SEM_CMD_RM, SHM_CMD_AT, SHM_CMD_DT, SHM_CMD_GET,
    SHM_CMD_INFO, SHM_CMD_RM,
};
pub use uring::queue::{
    enter_ring, setup_ring, CompletionQueueEntry, SubmissionQueueEntry, CQ_ENTRIES, SQ_ENTRIES,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shm_lifecycle_and_removal() {
        unsafe {
            // Allocate a new segment
            let shmid = create_shm(4096).expect("Failed to create SHM");
            assert!(shmid < 4);

            let frame = get_shm_frame(shmid).expect("Failed to get SHM frame");
            assert_eq!(frame, 0x70000000 + (shmid as u64 * 0x1000));

            let table = get_shm_table();
            assert!(table[shmid].in_use);
            assert_eq!(table[shmid].size_bytes, 4096);

            // Remove segment
            remove_shm(shmid as u32).expect("Failed to remove SHM");
            let table = get_shm_table();
            assert!(!table[shmid].in_use);
        }
    }

    #[test]
    fn test_sem_lifecycle_and_removal() {
        unsafe {
            let semid = create_sem(0x99998888, 7).expect("Failed to create semaphore");
            assert!(semid < 4);

            let table = get_sem_table();
            assert!(table[semid as usize].in_use);
            assert_eq!(table[semid as usize].value, 7);

            remove_sem(semid).expect("Failed to remove semaphore");
            let table = get_sem_table();
            assert!(!table[semid as usize].in_use);
        }
    }

    #[test]
    fn test_mqueue_send_receive_unlink() {
        unsafe {
            let qname = "/test_queue_prio";
            let mqid = mq_open(qname, 0, 8, 128).expect("Failed to open mqueue");
            assert!(mqid < MAX_MQUEUES as u32);

            // Send lower priority message first
            mq_send(qname, b"normal message", 5).expect("Send failed");
            // Send higher priority message second
            mq_send(qname, b"urgent message", 20).expect("Send failed");

            // Receive should return the higher priority message first
            let mut buf = [0u8; 128];
            let (len, prio) = mq_receive(qname, &mut buf).expect("Recv failed");
            assert_eq!(prio, 20);
            assert_eq!(&buf[..len], b"urgent message");

            // Next receive should return the lower priority message
            let (len2, prio2) = mq_receive(qname, &mut buf).expect("Recv failed");
            assert_eq!(prio2, 5);
            assert_eq!(&buf[..len2], b"normal message");

            // Queue should now be empty
            assert!(mq_receive(qname, &mut buf).is_err());

            // Unlink
            mq_unlink(qname).expect("Unlink failed");
        }
    }
}
