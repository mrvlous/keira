// SPDX-License-Identifier: GPL-2.0-only

//! Hardware Abstraction Layer (HAL) interfaces for Keira Kernel.
//!
//! Subdivided into dedicated traits for CPU execution, interrupt controllers,
//! Memory Management Units (MMU), serial communication and hardware timers.

pub mod cpu;
pub mod interrupt;
pub mod mmu;
pub mod serial;
pub mod timer;

pub use cpu::Cpu;
pub use interrupt::InterruptController;
pub use mmu::Mmu;
pub use serial::SerialPort;
pub use timer::Timer;
