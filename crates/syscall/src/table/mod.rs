// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! System call numbers (vector table) supported by Keira Kernel.

pub const SYS_PUTC: u64 = 1;
pub const SYS_EXIT: u64 = 2;
pub const SYS_SLEEP: u64 = 3;
pub const SYS_UPTIME: u64 = 4;
pub const SYS_EXEC: u64 = 5;
pub const SYS_OPEN: u64 = 6;
pub const SYS_READ: u64 = 7;
pub const SYS_WRITE: u64 = 8;
pub const SYS_CLOSE: u64 = 9;
pub const SYS_LSEEK: u64 = 10;
pub const SYS_BRK: u64 = 12;
pub const SYS_WAIT: u64 = 13;
pub const SYS_GETPID: u64 = 14;
pub const SYS_GETCWD: u64 = 15;
pub const SYS_CHDIR: u64 = 16;
pub const SYS_HTTP: u64 = 17;
pub const SYS_MMAP: u64 = 20;
pub const SYS_MUNMAP: u64 = 21;
pub const SYS_KILL: u64 = 22;
pub const SYS_PIPE: u64 = 23;
pub const SYS_SOCKET: u64 = 24;
pub const SYS_CONNECT: u64 = 25;
pub const SYS_FORK: u64 = 30;
pub const SYS_MPROTECT: u64 = 31;
pub const SYS_MADVISE: u64 = 32;
pub const SYS_TLS_CONNECT: u64 = 33;
pub const SYS_CLOCK_GETTIME_FAST: u64 = 36;
pub const SYS_FUTEX: u64 = 40;
pub const SYS_LISTEN: u64 = 50;
pub const SYS_ACCEPT: u64 = 43;
pub const SYS_SPLICE: u64 = 47;
pub const SYS_VMSPLICE: u64 = 48;
pub const SYS_EVENTFD: u64 = 50;
pub const SYS_SIGNALFD: u64 = 51;
pub const SYS_SECCOMP: u64 = 52;
pub const SYS_GETTIMEOFDAY: u64 = 53;
pub const SYS_SETTIMEOFDAY: u64 = 54;
pub const SYS_EPOLL_CREATE: u64 = 55;
pub const SYS_EPOLL_CTL: u64 = 56;
pub const SYS_EPOLL_WAIT: u64 = 57;
pub const SYS_MQ_OPEN: u64 = 58;
pub const SYS_PRCTL: u64 = 59;
pub const SYS_GETUID: u64 = 60;
pub const SYS_SETUID: u64 = 61;
pub const SYS_WAITPID: u64 = 62;
pub const SYS_GETPPID: u64 = 63;
pub const SYS_SIGACTION: u64 = 64;
pub const SYS_SIGRETURN: u64 = 65;
pub const SYS_CLOCK_GETTIME: u64 = 66;
pub const SYS_NANOSLEEP: u64 = 67;
pub const SYS_SYNC: u64 = 70;
pub const SYS_FSYNC: u64 = 71;
pub const SYS_FCNTL: u64 = 72;
pub const SYS_IOCTL: u64 = 73;
pub const SYS_RAID_LVM: u64 = 74;
pub const SYS_SHM_SEM: u64 = 75;
pub const SYS_NETFILTER: u64 = 76;
pub const SYS_PERF_EVENT: u64 = 77;
pub const SYS_BPF: u64 = 78;
pub const SYS_TPM2: u64 = 79;
pub const SYS_PCI_BRIDGE: u64 = 80;
