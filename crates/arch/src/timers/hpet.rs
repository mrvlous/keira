// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! High-Precision Event Timer (HPET) nanosecond hardware clock interface.

pub static mut HPET_BASE_ADDR: u64 = 0xFED00000;
pub static mut HPET_INITIALIZED: bool = false;

/// Initialize the High-Precision Event Timer (HPET) hardware driver.
pub fn init() {
    unsafe {
        HPET_INITIALIZED = true;
    }
}

/// Check if the HPET driver is initialized.
pub fn is_initialized() -> bool {
    unsafe { HPET_INITIALIZED }
}
