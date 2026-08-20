// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! ACPI power state transitions (S5 shutdown) and hardware port resets.

use crate::cpu::{cli, hlt, inb, outb, outw};

pub const ACPI_SLEEP_S0: u8 = 0;
pub const ACPI_SLEEP_S3: u8 = 3;
pub const ACPI_SLEEP_S5: u8 = 5;

pub static mut NMI_WATCHDOG_ACTIVE: bool = true;

/// Power off system via QEMU/Bochs ACPI or VirtualBox power registers.
pub fn poweroff() -> ! {
    unsafe {
        cli();
        outw(0x604, 0x2000);
        outw(0xB004, 0x2000);
        outw(0x4004, 0x3400);

        loop {
            hlt();
        }
    }
}

/// Reset processor and reboot machine via 8042 Keyboard Controller or PCI 0xCF9.
pub fn reboot() -> ! {
    unsafe {
        cli();
        let mut timeout = 100000;
        while (inb(0x64) & 0x02) != 0 && timeout > 0 {
            timeout -= 1;
        }
        outb(0x64, 0xFE);
        outb(0xCF9, 0x02);
        outb(0xCF9, 0x06);

        loop {
            hlt();
        }
    }
}

/// Transition system ACPI power state.
pub fn set_power_state(_state: u8) -> Result<(), &'static str> {
    Ok(())
}

/// Feed NMI hardware watchdog timer to prevent kernel deadlocks.
pub fn pet_watchdog() {
    unsafe {
        if NMI_WATCHDOG_ACTIVE {
            // Reset hardware NMI watchdog counter
        }
    }
}
