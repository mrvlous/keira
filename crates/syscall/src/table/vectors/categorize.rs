// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! System call vector categorization and validation predicates.

use crate::table::numbers::*;
use crate::table::vectors::names::syscall_name;

/// Checks whether a given syscall number is a valid recognized vector.
pub fn is_valid_syscall(num: u64) -> bool {
    syscall_name(num) != "sys_unknown"
}

/// Returns true if the syscall relates to file I/O operations.
pub fn is_io_syscall(num: u64) -> bool {
    matches!(
        num,
        SYS_OPEN
            | SYS_READ
            | SYS_WRITE
            | SYS_CLOSE
            | SYS_LSEEK
            | SYS_SYNC
            | SYS_FSYNC
            | SYS_FCNTL
            | SYS_IOCTL
            | SYS_DUP
            | SYS_DUP2
            | SYS_SPLICE
            | SYS_VMSPLICE
    )
}

/// Returns true if the syscall relates to memory management.
pub fn is_memory_syscall(num: u64) -> bool {
    matches!(
        num,
        SYS_SBRK | SYS_BRK | SYS_MMAP | SYS_MUNMAP | SYS_MPROTECT | SYS_MADVISE | SYS_MSYNC
    )
}

/// Returns true if the syscall relates to process lifecycle or task scheduling.
pub fn is_process_syscall(num: u64) -> bool {
    matches!(
        num,
        SYS_EXIT
            | SYS_EXEC
            | SYS_WAIT
            | SYS_GETPID
            | SYS_FORK
            | SYS_CLONE_THREAD
            | SYS_WAITPID
            | SYS_GETPPID
            | SYS_GETUID
            | SYS_SETUID
            | SYS_GETGID
            | SYS_SETGID
    )
}

/// Returns true if the syscall relates to inter-process communication (IPC).
pub fn is_ipc_syscall(num: u64) -> bool {
    matches!(
        num,
        SYS_PIPE
            | SYS_SHMGET
            | SYS_SHMAT
            | SYS_FUTEX
            | SYS_EVENTFD
            | SYS_SIGNALFD
            | SYS_EPOLL_CREATE
            | SYS_EPOLL_CTL
            | SYS_EPOLL_WAIT
            | SYS_MQ_OPEN
            | SYS_SHM_SEM
    )
}
