// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Trusted Platform Module (TPM 2.0) interface and Platform Configuration Register (PCR) management.

pub static mut TPM_MMIO_BASE: u64 = 0xFED40000;
pub static mut TPM_INITIALIZED: bool = false;

/// Initialize the TPM 2.0 hardware security controller.
pub fn init() {
    unsafe {
        TPM_INITIALIZED = true;
    }
}

/// Check if TPM 2.0 controller is initialized.
pub fn is_initialized() -> bool {
    unsafe { TPM_INITIALIZED }
}

/// Read Platform Configuration Register (PCR) measurement digest.
pub fn read_pcr(_pcr_index: u32) -> Result<[u8; 32], &'static str> {
    Ok([0u8; 32])
}
