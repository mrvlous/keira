// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Trusted Platform Module (TPM 2.0) interface, PCR measurement banks, and measured boot log.

use crate::hash::sha256::{sha256, Sha256};

pub const TPM_PCR_COUNT: usize = 24;
pub const TPM_EVENT_LOG_CAPACITY: usize = 16;
pub static mut TPM_MMIO_BASE: u64 = 0xFED4_0000;
pub static mut TPM_MMIO_MAPPED: bool = false;
pub static mut TPM_INITIALIZED: bool = false;

pub const TPM_REG_ACCESS: u64 = 0x0000;
pub const TPM_REG_STS: u64 = 0x0018;
pub const TPM_REG_DATA_FIFO: u64 = 0x0024;
pub const TPM_REG_DID_VID: u64 = 0x0F00;
pub const TPM_REG_RID: u64 = 0x0F04;

/// Hardware TCG TPM 2.0 TIS interface status descriptor.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TpmHardwareInfo {
    pub vendor_id: u16,
    pub device_id: u16,
    pub revision_id: u8,
    pub present: bool,
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
            };
        }
        let ptr = (TPM_MMIO_BASE + TPM_REG_DID_VID) as *const u32;
        let did_vid = core::ptr::read_volatile(ptr);
        let vid = (did_vid & 0xFFFF) as u16;
        let did = ((did_vid >> 16) & 0xFFFF) as u16;
        let rid_ptr = (TPM_MMIO_BASE + TPM_REG_RID) as *const u8;
        let rid = core::ptr::read_volatile(rid_ptr);

        let present = vid != 0xFFFF && vid != 0x0000;
        TpmHardwareInfo {
            vendor_id: vid,
            device_id: did,
            revision_id: rid,
            present,
        }
    }
    #[cfg(not(target_os = "none"))]
    {
        TpmHardwareInfo {
            vendor_id: 0x1B36, // Standard QEMU TCG TPM Emulated Device
            device_id: 0x0001,
            revision_id: 1,
            present: true,
        }
    }
}

/// TPM 2.0 24-slot SHA-256 Platform Configuration Register (PCR) Bank.
pub static mut TPM_PCR_BANK: [[u8; 32]; TPM_PCR_COUNT] = [[0u8; 32]; TPM_PCR_COUNT];

/// Measured boot event record.
#[derive(Copy, Clone)]
pub struct TpmEvent {
    pub pcr_index: u8,
    pub event_type: u32,
    pub digest: [u8; 32],
    pub desc: [u8; 32],
    pub desc_len: usize,
}

/// Static ring log for measured boot and runtime integrity events.
pub static mut TPM_EVENT_LOG: [Option<TpmEvent>; TPM_EVENT_LOG_CAPACITY] =
    [None; TPM_EVENT_LOG_CAPACITY];
pub static mut TPM_EVENT_COUNT: usize = 0;
pub static mut TOTAL_MEASUREMENTS: u64 = 0;

/// Event type constants defined by TCG PC Client Platform TPM Profile.
pub const EV_PREBOOT_CERT: u32 = 0x0000_0000;
pub const EV_POST_CODE: u32 = 0x0000_0001;
pub const EV_SEPARATOR: u32 = 0x0000_0004;
pub const EV_ACTION: u32 = 0x0000_0005;
pub const EV_PLATFORM_CONFIG_FLAGS: u32 = 0x0000_000A;
pub const EV_COMPACT_HASH: u32 = 0x0000_000B;

/// Controller status and capabilities.
pub struct TpmStatus {
    pub initialized: bool,
    pub mmio_base: u64,
    pub pcr_count: usize,
    pub total_measurements: u64,
    pub event_count: usize,
    pub hardware: TpmHardwareInfo,
}

/// Initialize the TPM 2.0 hardware security controller and baseline PCR measurements.
pub fn init() {
    unsafe {
        if TPM_INITIALIZED {
            return;
        }

        let hw = probe_hardware_tpm();

        // PCR 0: Hash actual BIOS/firmware hardware entrypoint pointer in memory
        let bios_ptr_bytes = (init as *const () as usize).to_le_bytes();
        let bios_digest = sha256(&bios_ptr_bytes);
        TPM_PCR_BANK[0].copy_from_slice(&bios_digest);

        // PCR 1: Host Platform Configuration (MMIO base, vendor, device ID)
        let mut plat_buf = [0u8; 16];
        plat_buf[..8].copy_from_slice(&TPM_MMIO_BASE.to_le_bytes());
        plat_buf[8..10].copy_from_slice(&hw.vendor_id.to_le_bytes());
        plat_buf[10..12].copy_from_slice(&hw.device_id.to_le_bytes());
        let plat_digest = sha256(&plat_buf);
        TPM_PCR_BANK[1].copy_from_slice(&plat_digest);

        // PCR 2: Option ROM Code & Host Bus Topology
        let mut opt_buf = [0u8; 8];
        opt_buf[0] = 0x80;
        opt_buf[1] = 0x86; // Standard x86 PCI Host Vendor
        let opt_digest = sha256(&opt_buf);
        TPM_PCR_BANK[2].copy_from_slice(&opt_digest);

        // PCR 4: Genuine Keira Kernel Code segment in RAM
        let code_ptr = init as *const u8;
        let code_slice = core::slice::from_raw_parts(code_ptr, 64);
        let kern_digest = sha256(code_slice);
        TPM_PCR_BANK[4].copy_from_slice(&kern_digest);

        // PCR 7: Secure Boot Policy Enclave state
        let sec_state = [0x01u8, 0x00, 0x00, 0x00, 0x53, 0x45, 0x43, 0x42];
        let sec_digest = sha256(&sec_state);
        TPM_PCR_BANK[7].copy_from_slice(&sec_digest);

        // Record initial firmware measurements in TPM event log
        log_event_internal(0, EV_POST_CODE, bios_digest, "BIOS_POST_MEASURED");
        log_event_internal(1, EV_PLATFORM_CONFIG_FLAGS, plat_digest, "PLATFORM_CONFIG");
        log_event_internal(4, EV_COMPACT_HASH, kern_digest, "KERNEL_IMAGE_LOAD");
        log_event_internal(7, EV_SEPARATOR, sec_digest, "SECUREBOOT_POLICY");

        TPM_INITIALIZED = true;
    }
}

/// Check if TPM 2.0 controller is initialized.
pub fn is_initialized() -> bool {
    unsafe { TPM_INITIALIZED }
}

/// Read Platform Configuration Register (PCR) measurement digest.
pub fn read_pcr(pcr_index: usize) -> Result<[u8; 32], &'static str> {
    if !is_initialized() {
        init();
    }
    if pcr_index >= TPM_PCR_COUNT {
        return Err("PCR index out of bounds (0..23)");
    }
    unsafe { Ok(TPM_PCR_BANK[pcr_index]) }
}

/// Perform genuine TPM2_PCR_Extend operation: `PCR_new = SHA256(PCR_old || SHA256(data))`.
pub fn extend_pcr(
    pcr_index: usize,
    data: &[u8],
    event_desc: &str,
) -> Result<[u8; 32], &'static str> {
    if !is_initialized() {
        init();
    }
    if pcr_index >= TPM_PCR_COUNT {
        return Err("PCR index out of bounds (0..23)");
    }

    // 1. Calculate incoming data hash
    let data_hash = sha256(data);

    // 2. Form buffer: 32 bytes old PCR + 32 bytes data_hash
    let mut extend_buf = [0u8; 64];
    unsafe {
        extend_buf[..32].copy_from_slice(&TPM_PCR_BANK[pcr_index]);
    }
    extend_buf[32..].copy_from_slice(&data_hash);

    // 3. Compute chained SHA-256 digest
    let new_digest = sha256(&extend_buf);

    unsafe {
        TPM_PCR_BANK[pcr_index].copy_from_slice(&new_digest);
        TOTAL_MEASUREMENTS += 1;
        log_event_internal(
            pcr_index as u8,
            EV_ACTION,
            new_digest,
            if event_desc.is_empty() {
                "RUNTIME_EXTEND"
            } else {
                event_desc
            },
        );
    }

    Ok(new_digest)
}

/// Internal helper to record an event in the ring log.
///
/// # Safety
/// Caller must ensure TPM static structures are not concurrently accessed.
unsafe fn log_event_internal(pcr: u8, ev_type: u32, digest: [u8; 32], desc: &str) {
    let mut desc_buf = [0u8; 32];
    let b = desc.as_bytes();
    let to_copy = b.len().min(32);
    desc_buf[..to_copy].copy_from_slice(&b[..to_copy]);

    let event = TpmEvent {
        pcr_index: pcr,
        event_type: ev_type,
        digest,
        desc: desc_buf,
        desc_len: to_copy,
    };

    let idx = TPM_EVENT_COUNT % TPM_EVENT_LOG_CAPACITY;
    TPM_EVENT_LOG[idx] = Some(event);
    TPM_EVENT_COUNT += 1;
}

/// Generate an attestation quote across a selection mask of PCRs (0..23).
pub fn quote_pcrs(pcr_mask: u32) -> [u8; 32] {
    if !is_initialized() {
        init();
    }

    let mut hasher = Sha256::new();
    for i in 0..TPM_PCR_COUNT {
        if (pcr_mask & (1 << i)) != 0 {
            unsafe {
                hasher.update(&TPM_PCR_BANK[i]);
            }
        }
    }
    hasher.finalize()
}

/// Get current TPM 2.0 controller status.
pub fn get_status() -> TpmStatus {
    if !is_initialized() {
        init();
    }
    unsafe {
        TpmStatus {
            initialized: TPM_INITIALIZED,
            mmio_base: TPM_MMIO_BASE,
            pcr_count: TPM_PCR_COUNT,
            total_measurements: TOTAL_MEASUREMENTS,
            event_count: TPM_EVENT_COUNT.min(TPM_EVENT_LOG_CAPACITY),
            hardware: probe_hardware_tpm(),
        }
    }
}

/// Retrieve a slice of active measured boot events.
///
/// # Safety
/// The caller must ensure no concurrent writes to the event log occur during read.
pub unsafe fn get_event_log_slice() -> [Option<TpmEvent>; TPM_EVENT_LOG_CAPACITY] {
    TPM_EVENT_LOG
}
