// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! System call numbers (vector table) for all 62 system calls supported by Keira Kernel.

pub const SYS_PUTC: u64 = 1;
pub const SYS_EXIT: u64 = 2;
pub const SYS_SLEEP: u64 = 3;
pub const SYS_UPTIME: u64 = 4;
pub const SYS_EXEC: u64 = 5;
pub const SYS_BRK: u64 = 12;
pub const SYS_OPEN: u64 = 14;
pub const SYS_READ: u64 = 15;
pub const SYS_WRITE: u64 = 16;
pub const SYS_CLOSE: u64 = 17;
pub const SYS_LSEEK: u64 = 18;
pub const SYS_LIST: u64 = 19;
pub const SYS_GETPID: u64 = 20;
pub const SYS_FORK: u64 = 21;
pub const SYS_KILL: u64 = 22;
pub const SYS_PIPE: u64 = 23;
pub const SYS_DUP: u64 = 24;
pub const SYS_DUP2: u64 = 25;
pub const SYS_UNLINK: u64 = 26;
pub const SYS_MKDIR: u64 = 27;
pub const SYS_SHMGET: u64 = 28;
pub const SYS_SHMAT: u64 = 29;
pub const SYS_MMAP: u64 = 30;
pub const SYS_MUNMAP: u64 = 31;
pub const SYS_FUTEX: u64 = 32;
pub const SYS_GETCWD: u64 = 33;
pub const SYS_CHDIR: u64 = 34;
pub const SYS_STAT: u64 = 35;
pub const SYS_RMDIR: u64 = 36;
pub const SYS_SOCKET: u64 = 41;
pub const SYS_CONNECT: u64 = 42;
pub const SYS_SENDTO: u64 = 44;
pub const SYS_RECVFROM: u64 = 45;
pub const SYS_BIND: u64 = 49;
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
