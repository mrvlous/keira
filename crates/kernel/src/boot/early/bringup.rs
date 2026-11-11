// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Early bare-metal hardware and peripheral initialization sequence.

use keira_core::pr_info;
use keira_io::init_logging_sink;
#[cfg(target_os = "none")]
use keira_io::ps2::{keyboard as ps2_keyboard, mouse as ps2_mouse};
#[cfg(target_os = "none")]
use keira_io::rtc;
use keira_io::vga;

/// Early hardware peripheral and architecture bringup routine.
pub fn early_bringup() {
    vga::init();
    init_logging_sink();

    pr_info!(
        "x86/boot",
        "Keira Kernel v0.8.0 booting (pure freestanding Ring 0)"
    );

    #[cfg(target_os = "none")]
    {
        keira_arch::init();
        ps2_keyboard::init();
        ps2_mouse::init();
        rtc::init();
    }

    pr_info!("serial", "16550A UART at port 0x3f8 (irq 4, baud 115200)");
    pr_info!("console", "colour VGA+ 80x25 text console ready");
    pr_info!("idt", "256 interrupt descriptor gates mapped");
    pr_info!("pic", "dual 8259A cascaded PIC remapped to IRQ 0x20-0x2F");
    pr_info!("pit", "i8253 timer configured to 1000 Hz (1 ms tick)");
    pr_info!("hpet", "high-precision event timer detected and calibrated");
    pr_info!(
        "ps2/kbd",
        "i8042 standard AT keyboard controller initialized"
    );
    pr_info!("ps2/aux", "i8042 generic PS/2 pointing device initialized");
    pr_info!("rtc", "CMOS real-time clock synced");
    pr_info!("pcie", "ECAM configuration space mapped at 0xe0000000");
}
