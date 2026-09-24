// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! High-level Interrupt Service Routine (ISR) handlers and dispatchers.

use super::super::pic;

/// Generic ISR dispatcher called from low-level assembly stubs.
#[no_mangle]
pub extern "C" fn isr_handler(vector: usize) {
    if vector == 32 {
        crate::timers::pit::pit_handler();
    } else if vector >= 40 {
        pic::send_eoi(8);
    } else {
        pic::send_eoi(0);
    }
}
