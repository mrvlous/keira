// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! System call name lookup, vector categorization and validation predicates.

use crate::table::numbers::*;

/// Returns the symbolic name for a given system call number.
pub fn syscall_name(num: u64) -> &'static str {
    match num {
        SYS_PUTC => "sys_putc",
        SYS_EXIT => "sys_exit",
        SYS_SLEEP => "sys_sleep",
        SYS_UPTIME => "sys_uptime",
        SYS_EXEC => "sys_exec",
        SYS_OPEN => "sys_open",
        SYS_READ => "sys_read",
        SYS_WRITE => "sys_write",
        SYS_CLOSE => "sys_close",
        SYS_LSEEK => "sys_lseek",
        SYS_SBRK => "sys_sbrk",
        SYS_BRK => "sys_brk",
        SYS_WAIT => "sys_wait",
        SYS_GETPID => "sys_getpid",
        SYS_GETCWD => "sys_getcwd",
        SYS_CHDIR => "sys_chdir",
        SYS_HTTP => "sys_http",
        SYS_MMAP => "sys_mmap",
        SYS_MUNMAP => "sys_munmap",
        SYS_KILL => "sys_kill",
        SYS_PIPE => "sys_pipe",
        SYS_SOCKET => "sys_socket",
        SYS_CONNECT => "sys_connect",
        SYS_SHMGET => "sys_shmget",
        SYS_SHMAT => "sys_shmat",
        SYS_FORK => "sys_fork",
        SYS_MPROTECT => "sys_mprotect",
        SYS_MADVISE => "sys_madvise",
        SYS_TLS_CONNECT => "sys_tls_connect",
        SYS_INIT_MODULE => "sys_init_module",
        SYS_DELETE_MODULE => "sys_delete_module",
        SYS_CLOCK_GETTIME_FAST => "sys_clock_gettime_fast",
        SYS_PTRACE => "sys_ptrace",
        SYS_IO_URING_SETUP => "sys_io_uring_setup",
        SYS_IO_URING_ENTER => "sys_io_uring_enter",
        SYS_FUTEX => "sys_futex",
        SYS_CLONE_THREAD => "sys_clone_thread",
        SYS_KVM_CREATE_VM => "sys_kvm_create_vm",
        SYS_SYSLOG => "sys_syslog",
        SYS_TIMER_CREATE => "sys_timer_create",
        SYS_TIMER_SETTIME => "sys_timer_settime",
        SYS_SPLICE => "sys_splice",
        SYS_VMSPLICE => "sys_vmsplice",
        SYS_PERF_EVENT_OPEN => "sys_perf_event_open",
        SYS_EVENTFD => "sys_eventfd",
        SYS_SIGNALFD => "sys_signalfd",
        SYS_SECCOMP => "sys_seccomp",
        SYS_GETTIMEOFDAY => "sys_gettimeofday",
        SYS_SETTIMEOFDAY => "sys_settimeofday",
        SYS_EPOLL_CREATE => "sys_epoll_create",
        SYS_EPOLL_CTL => "sys_epoll_ctl",
        SYS_EPOLL_WAIT => "sys_epoll_wait",
        SYS_MQ_OPEN => "sys_mq_open",
        SYS_PRCTL => "sys_prctl",
        SYS_GETUID => "sys_getuid",
        SYS_SETUID => "sys_setuid",
        SYS_WAITPID => "sys_waitpid",
        SYS_GETPPID => "sys_getppid",
        SYS_SIGACTION => "sys_sigaction",
        SYS_SIGRETURN => "sys_sigreturn",
        SYS_CLOCK_GETTIME => "sys_clock_gettime",
        SYS_NANOSLEEP => "sys_nanosleep",
        SYS_GETGID => "sys_getgid",
        SYS_SETGID => "sys_setgid",
        SYS_SYNC => "sys_sync",
        SYS_FSYNC => "sys_fsync",
        SYS_FCNTL => "sys_fcntl",
        SYS_IOCTL => "sys_ioctl",
        SYS_RAID_LVM => "sys_raid_lvm",
        SYS_SHM_SEM => "sys_shm_sem",
        SYS_NETFILTER => "sys_netfilter",
        SYS_PERF_EVENT => "sys_perf_event",
        SYS_BPF => "sys_bpf",
        SYS_TPM2 => "sys_tpm2",
        SYS_PCI_BRIDGE => "sys_pci_bridge",
        SYS_SIGPROCMASK => "sys_sigprocmask",
        SYS_SIGPENDING => "sys_sigpending",
        SYS_MSYNC => "sys_msync",
        SYS_DUP => "sys_dup",
        SYS_DUP2 => "sys_dup2",
        SYS_GETDENTS => "sys_getdents",
        SYS_UNLINK => "sys_unlink",
        SYS_MKDIR => "sys_mkdir",
        SYS_PING => "sys_ping",
        SYS_GETEUID => "sys_geteuid",
        SYS_GETEGID => "sys_getegid",
        SYS_SYMLINK => "sys_symlink",
        SYS_READLINK => "sys_readlink",
        SYS_MOUNT => "sys_mount",
        SYS_UMOUNT => "sys_umount",
        SYS_PIPE2 => "sys_pipe2",
        SYS_CHMOD => "sys_chmod",
        SYS_CHOWN => "sys_chown",
        SYS_STAT => "sys_stat",
        SYS_FSTAT => "sys_fstat",
        SYS_GETRANDOM => "sys_getrandom",
        SYS_GETRLIMIT => "sys_getrlimit",
        SYS_SETRLIMIT => "sys_setrlimit",
        SYS_PRLIMIT64 => "sys_prlimit64",
        SYS_SOCKETPAIR => "sys_socketpair",
        SYS_POLL => "sys_poll",
        SYS_SELECT => "sys_select",
        SYS_UNAME => "sys_uname",
        SYS_REBOOT => "sys_reboot",
        SYS_DNS_RESOLVE => "sys_dns_resolve",
        _ => "sys_unknown",
    }
}

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
            | SYS_GETDENTS
            | SYS_UNLINK
            | SYS_MKDIR
            | SYS_SPLICE
            | SYS_VMSPLICE
            | SYS_SYMLINK
            | SYS_READLINK
            | SYS_MOUNT
            | SYS_UMOUNT
            | SYS_CHMOD
            | SYS_CHOWN
            | SYS_STAT
            | SYS_FSTAT
            | SYS_POLL
            | SYS_SELECT
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
            | SYS_GETEUID
            | SYS_GETEGID
            | SYS_GETRLIMIT
            | SYS_SETRLIMIT
            | SYS_PRLIMIT64
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
