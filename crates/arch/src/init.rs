// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Early x86_64 Architecture Hardware Bringup and Subsystem Initialization.

use crate::interrupts::{idt, pic};
use crate::timers::pit;

/// Initialize core CPU architectural structures (IDT, Dual 8259 PIC, and 1000Hz PIT timer).
pub fn init() {
    // 1. Initialize Interrupt Descriptor Table
    idt::init();

    // 2. Initialize and remap Dual 8259 PIC to IRQ 32 and 40
    pic::init(32, 40);

    // 3. Configure 8254 PIT timer to 1000Hz (1ms tick interval)
    pit::init(1000);
}
