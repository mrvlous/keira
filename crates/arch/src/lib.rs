// SPDX-License-Identifier: GPL-2.0-only

//! x86_64 CPU instructions, registers, timers, APIC, power and virtualization for Keira Kernel.

#![no_std]

pub mod cpu;
pub mod debug;
pub mod hal;
pub mod init;
pub mod interrupts;
pub mod perf;
pub mod power;
pub mod timers;
pub mod virt;

pub use cpu::*;
pub use debug::unwind;
pub use debug::*;
pub use hal::*;
pub use init::init;
pub use interrupts::*;
pub use perf::pmu as perf_pmu;
pub use perf::*;
pub use power::acpi as power_acpi;
pub use power::*;
pub use timers::posix as timer;
pub use timers::*;
pub use virt::kvm;
pub use virt::*;

#[cfg(test)]
mod tests;
