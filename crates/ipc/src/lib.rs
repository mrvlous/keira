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
pub use mqueue::queue::{self as mqueue_queue, sys_mq_open, MessageQueue};
pub use pipe::fifo::{
    create_pipe, read_pipe, write_pipe, PipeBuffer, PIPE_BUFFER_SIZE, SYSTEM_PIPE,
};
pub use pipe::splice::{sys_splice, sys_vmsplice};
pub use shm::segment::{
    create_shm, get_shm_frame, sys_shm_sem, Semaphore, ShmSegment, SHM_CMD_AT, SHM_CMD_DT,
    SHM_CMD_GET, SHM_CMD_INFO, SHM_CMD_RM,
};
pub use uring::queue::{
    enter_ring, setup_ring, CompletionQueueEntry, SubmissionQueueEntry, CQ_ENTRIES, SQ_ENTRIES,
};
