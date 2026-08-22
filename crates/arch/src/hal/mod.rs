// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardware Abstraction Layer (HAL) interfaces for Keira Kernel.

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
