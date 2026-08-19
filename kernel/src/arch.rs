// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! Architecture-specific kernel subsystem modules, CPU features,
//! hardware timers, APIC controllers, and hypervisor primitives.

#[path = "../../arch/x86/kernel/apic.rs"]
pub mod apic;

/// High-Precision Event Timer (HPET) Subsystem
pub mod hpet;

/// Kernel Callstack Unwinder & Debugging Engine
pub mod unwind;

/// Symmetric Multiprocessing (SMP) IPI & TLB Shootdown Subsystem
pub mod smp;

/// Hardware Virtualization Hypervisor Subsystem (Intel VMX / AMD SVM)
pub mod kvm;

/// High-Resolution POSIX Interval Timers Engine
pub mod timer;

/// ACPI Hardware Power Management & NMI Watchdog Subsystem
pub mod power;

/// Hardware Performance Counters & PMU Engine
pub mod perf;

#[cfg(target_arch = "x86_64")]
pub const ARCH_NAME: &str = "x86_64";

#[cfg(target_arch = "aarch64")]
pub const ARCH_NAME: &str = "aarch64";

#[cfg(target_arch = "riscv64")]
pub const ARCH_NAME: &str = "riscv64";
