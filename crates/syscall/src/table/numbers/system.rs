// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! System hardware, timing, virtualization, and cryptographic coprocessor syscall numbers.

pub const SYS_INIT_MODULE: u64 = 34;
pub const SYS_DELETE_MODULE: u64 = 35;
pub const SYS_CLOCK_GETTIME_FAST: u64 = 36;
pub const SYS_PTRACE: u64 = 37;
pub const SYS_KVM_CREATE_VM: u64 = 42;
pub const SYS_KVM_RUN_VCPU: u64 = 43;
pub const SYS_SYSLOG: u64 = 44;
pub const SYS_TIMER_CREATE: u64 = 45;
pub const SYS_TIMER_SETTIME: u64 = 46;
pub const SYS_PERF_EVENT_OPEN: u64 = 49;
pub const SYS_SECCOMP: u64 = 52;
pub const SYS_GETTIMEOFDAY: u64 = 53;
pub const SYS_SETTIMEOFDAY: u64 = 54;
pub const SYS_CLOCK_GETTIME: u64 = 66;
pub const SYS_NANOSLEEP: u64 = 67;
pub const SYS_RAID_LVM: u64 = 74;
pub const SYS_PERF_EVENT: u64 = 77;
pub const SYS_BPF: u64 = 78;
pub const SYS_TPM2: u64 = 79;
pub const SYS_PCI_BRIDGE: u64 = 80;
