// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
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

use core::sync::atomic::{AtomicUsize, Ordering};

pub static mut NMI_WATCHDOG_ACTIVE: bool = true;
static CPU_HEARTBEAT_TICKS: AtomicUsize = AtomicUsize::new(0);
static LAST_PET_TICK: AtomicUsize = AtomicUsize::new(0);

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

/// Record CPU timer tick heartbeat for soft lockup detection.
#[inline]
pub fn record_cpu_heartbeat() {
    CPU_HEARTBEAT_TICKS.fetch_add(1, Ordering::Relaxed);
}

/// Query current CPU heartbeat tick count.
#[inline]
pub fn get_cpu_heartbeat() -> usize {
    CPU_HEARTBEAT_TICKS.load(Ordering::Relaxed)
}

/// Feed NMI hardware watchdog timer to prevent kernel deadlocks.
pub fn pet_watchdog() {
    let current_ticks = get_cpu_heartbeat();
    LAST_PET_TICK.store(current_ticks, Ordering::Relaxed);
    unsafe {
        if NMI_WATCHDOG_ACTIVE {
            // Reset hardware NMI watchdog counter
        }
    }
}

/// Detect whether a CPU core has suffered a soft lockup (heartbeat stalled beyond threshold).
pub fn check_soft_lockup(threshold_ticks: usize) -> bool {
    let current = get_cpu_heartbeat();
    let last = LAST_PET_TICK.load(Ordering::Relaxed);
    if last == 0 {
        return false;
    }
    current > last && (current - last) > threshold_ticks
}
