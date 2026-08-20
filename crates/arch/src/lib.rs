// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![no_std]

//! x86_64 CPU instructions, registers, timers, APIC, power, and virtualization for Keira Kernel.

pub mod cpu;
pub mod debug;
pub mod interrupts;
pub mod perf;
pub mod power;
pub mod timers;
pub mod virt;

pub use cpu::*;
pub use debug::unwind;
pub use debug::*;
pub use interrupts::*;
pub use perf::pmu as perf_pmu;
pub use perf::*;
pub use power::acpi as power_acpi;
pub use power::*;
pub use timers::posix as timer;
pub use timers::*;
pub use virt::kvm;
pub use virt::*;
