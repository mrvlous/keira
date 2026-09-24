// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Inter-Process Communication (Pipes, Futex, Epoll, EventFD, MQueue, Splice, POSIX SHM, io_uring).

#![no_std]
#![allow(static_mut_refs)]

pub mod event;
pub mod futex;
pub mod mqueue;
pub mod pipe;
pub mod shm;
pub mod uring;

#[cfg(test)]
mod tests;

pub use event::epoll::{
    self, check_fd_readiness, epoll_ctl_internal, epoll_reset, get_epoll_instances,
    get_epoll_stats, sys_epoll_create, sys_epoll_ctl, sys_epoll_wait, EpollEvent, EpollInstance,
    EpollItem, EPOLLERR, EPOLLET, EPOLLHUP, EPOLLIN, EPOLLOUT, EPOLLPRI, EPOLL_CTL_ADD,
    EPOLL_CTL_DEL, EPOLL_CTL_MOD, MAX_EPOLL_INSTANCES, MAX_EPOLL_ITEMS,
};
pub use event::eventfd::{
    self, close_eventfd, create_eventfd, get_eventfd_stats, get_eventfd_table, read_eventfd,
    sys_eventfd, sys_signalfd, write_eventfd, EventFd, EventFdEntry, EFD_CLOEXEC, EFD_NONBLOCK,
    EFD_SEMAPHORE, MAX_EVENTFDS,
};
pub use futex::sync::{
    self as futex_sync, cleanup_futex_waiters_for_pid, futex_requeue, futex_reset, futex_wait,
    futex_wake, get_futex_stats, get_futex_table, sys_futex, FutexWaiter, FUTEX_CMP_REQUEUE,
    FUTEX_CMP_REQUEUE_PI, FUTEX_FD, FUTEX_LOCK_PI, FUTEX_REQUEUE, FUTEX_TRYLOCK_PI,
    FUTEX_UNLOCK_PI, FUTEX_WAIT, FUTEX_WAIT_BITSET, FUTEX_WAIT_REQUEUE_PI, FUTEX_WAKE,
    FUTEX_WAKE_BITSET, FUTEX_WAKE_OP, MAX_FUTEX_WAITERS,
};
pub use mqueue::queue::{
    self as mqueue_queue, find_queue_mut, get_mqueue_stats, get_mqueue_table, mq_open, mq_receive,
    mq_send, mq_unlink, mq_unlink_by_id, sys_mq_open, MQueueMessage, PosixMessageQueue,
    MAX_MQUEUES, MQUEUE_MAX_MSGS, MQUEUE_MSG_SIZE,
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
    enter_ring, enter_ring_ext, get_ring, get_ring_mut, setup_ring, setup_ring_ext,
    CompletionQueueEntry, IoCqringOffsets, IoSqringOffsets, IoUringInstance, IoUringParams,
    IoUringRing, SubmissionQueueEntry, CQ_ENTRIES, IORING_ENTER_GETEVENTS, IORING_ENTER_SQ_WAIT,
    IORING_ENTER_SQ_WAKEUP, IORING_FEAT_NODROP, IORING_FEAT_SINGLE_MMAP, IORING_FEAT_SUBMIT_STABLE,
    IORING_OP_CLOSE, IORING_OP_FSYNC, IORING_OP_NOP, IORING_OP_POLL_ADD, IORING_OP_POLL_REMOVE,
    IORING_OP_READ, IORING_OP_READV, IORING_OP_READ_FIXED, IORING_OP_RECVMSG, IORING_OP_SENDMSG,
    IORING_OP_STATX, IORING_OP_SYNC_FILE_RANGE, IORING_OP_TIMEOUT, IORING_OP_WRITE,
    IORING_OP_WRITEV, IORING_OP_WRITE_FIXED, IORING_SETUP_ATTACH_WQ, IORING_SETUP_CLAMP,
    IORING_SETUP_CQSIZE, IORING_SETUP_IOPOLL, IORING_SETUP_SQPOLL, IORING_SETUP_SQ_AFF,
    IOSQE_ASYNC, IOSQE_BUFFER_SELECT, IOSQE_FIXED_FILE, IOSQE_IO_DRAIN, IOSQE_IO_HARDLINK,
    IOSQE_IO_LINK, MAX_IO_URING_INSTANCES, MAX_RING_ENTRIES, RINGS, SQ_ENTRIES,
};
