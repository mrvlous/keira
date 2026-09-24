// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Trusted Platform Module (TPM 2.0) TCG TIS interface and MMIO hardware registers.

#![allow(static_mut_refs)]

pub static mut TPM_MMIO_BASE: u64 = 0xFED4_0000;
pub static mut TPM_MMIO_MAPPED: bool = false;
pub static mut TPM_INITIALIZED: bool = false;

pub const TPM_REG_ACCESS: u64 = 0x0000;
pub const TPM_REG_STS: u64 = 0x0018;
pub const TPM_REG_DATA_FIFO: u64 = 0x0024;
pub const TPM_REG_DID_VID: u64 = 0x0F00;
pub const TPM_REG_RID: u64 = 0x0F04;

pub const TPM_REG_ACCESS_REQUEST_USE: u8 = 0x02;
pub const TPM_REG_ACCESS_ACTIVE_LOCALITY: u8 = 0x20;
pub const TPM_REG_ACCESS_VALID: u8 = 0x80;

/// Hardware TCG TPM 2.0 TIS interface status descriptor.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TpmHardwareInfo {
    pub vendor_id: u16,
    pub device_id: u16,
    pub revision_id: u8,
    pub present: bool,
    pub locality_active: bool,
}

/// Request TIS Locality via MMIO TPM_ACCESS register.
pub fn tis_request_locality(locality: u8) -> bool {
    #[cfg(target_os = "none")]
    unsafe {
        if !TPM_MMIO_MAPPED {
            return false;
        }
        let offset = (locality as u64) * 0x1000;
        let access_reg = (TPM_MMIO_BASE + offset + TPM_REG_ACCESS) as *mut u8;
        core::ptr::write_volatile(access_reg, TPM_REG_ACCESS_REQUEST_USE);
        for _ in 0..1000 {
            let val = core::ptr::read_volatile(access_reg);
            if (val & TPM_REG_ACCESS_ACTIVE_LOCALITY) != 0 {
                return true;
            }
        }
        false
    }
    #[cfg(not(target_os = "none"))]
    {
        let _ = locality;
        true
    }
}

/// Release TIS Locality via MMIO TPM_ACCESS register.
pub fn tis_release_locality(locality: u8) {
    #[cfg(target_os = "none")]
    unsafe {
        if !TPM_MMIO_MAPPED {
            return;
        }
        let offset = (locality as u64) * 0x1000;
        let access_reg = (TPM_MMIO_BASE + offset + TPM_REG_ACCESS) as *mut u8;
        core::ptr::write_volatile(access_reg, TPM_REG_ACCESS_ACTIVE_LOCALITY);
    }
    #[cfg(not(target_os = "none"))]
    {
        let _ = locality;
    }
}

/// Probe hardware MMIO registers for TCG TPM 2.0 TIS interface at base 0xFED4_0000.
pub fn probe_hardware_tpm() -> TpmHardwareInfo {
    #[cfg(target_os = "none")]
    unsafe {
        if !TPM_MMIO_MAPPED {
            return TpmHardwareInfo {
                vendor_id: 0,
                device_id: 0,
                revision_id: 0,
                present: false,
                locality_active: false,
            };
        }
        let ptr = (TPM_MMIO_BASE + TPM_REG_DID_VID) as *const u32;
        let did_vid = core::ptr::read_volatile(ptr);
        let vid = (did_vid & 0xFFFF) as u16;
        let did = ((did_vid >> 16) & 0xFFFF) as u16;
        let rid_ptr = (TPM_MMIO_BASE + TPM_REG_RID) as *const u8;
        let rid = core::ptr::read_volatile(rid_ptr);

        let present = vid != 0xFFFF && vid != 0x0000;
        let locality_active = if present {
            tis_request_locality(0)
        } else {
            false
        };

        TpmHardwareInfo {
            vendor_id: vid,
            device_id: did,
            revision_id: rid,
            present,
            locality_active,
        }
    }
    #[cfg(not(target_os = "none"))]
    {
        TpmHardwareInfo {
            vendor_id: 0x1B36,
            device_id: 0x0001,
            revision_id: 1,
            present: true,
            locality_active: true,
        }
    }
}

/// Controller status and capabilities.
pub struct TpmStatus {
    pub initialized: bool,
    pub mmio_base: u64,
    pub pcr_count: usize,
    pub total_measurements: u64,
    pub event_count: usize,
    pub hardware: TpmHardwareInfo,
}

/// Check if TPM 2.0 controller is initialized.
pub fn is_initialized() -> bool {
    unsafe { TPM_INITIALIZED }
}

/// Get current TPM 2.0 controller status.
pub fn get_status() -> TpmStatus {
    if !is_initialized() {
        crate::tpm::tpm2::init();
    }
    unsafe {
        TpmStatus {
            initialized: TPM_INITIALIZED,
            mmio_base: TPM_MMIO_BASE,
            pcr_count: super::pcr::TPM_PCR_COUNT,
            total_measurements: super::pcr::TOTAL_MEASUREMENTS,
            event_count: super::pcr::TPM_EVENT_COUNT.min(super::pcr::TPM_EVENT_LOG_CAPACITY),
            hardware: probe_hardware_tpm(),
        }
    }
}
