// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Platform Configuration Registers (PCR) and measured boot event logging.

#![allow(static_mut_refs)]

use super::hardware::{is_initialized, probe_hardware_tpm, TPM_INITIALIZED, TPM_MMIO_BASE};
use crate::hash::sha256::{sha256, Sha256};

pub const TPM_PCR_COUNT: usize = 24;
pub const TPM_EVENT_LOG_CAPACITY: usize = 32;

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

/// Initialize the TPM 2.0 hardware security controller and baseline PCR measurements.
pub fn init() {
    unsafe {
        if TPM_INITIALIZED {
            return;
        }

        let hw = probe_hardware_tpm();

        #[cfg(target_os = "none")]
        let bios_digest = {
            let bda_slice = core::slice::from_raw_parts(0x0000_0400 as *const u8, 256);
            sha256(bda_slice)
        };
        #[cfg(not(target_os = "none"))]
        let bios_digest = sha256(b"KEIRA_BIOS_EMULATED_IVT_SIGNATURE");

        TPM_PCR_BANK[0].copy_from_slice(&bios_digest);

        let mut plat_buf = [0u8; 16];
        plat_buf[..8].copy_from_slice(&TPM_MMIO_BASE.to_le_bytes());
        plat_buf[8..10].copy_from_slice(&hw.vendor_id.to_le_bytes());
        plat_buf[10..12].copy_from_slice(&hw.device_id.to_le_bytes());
        plat_buf[12] = if hw.locality_active { 1 } else { 0 };
        let plat_digest = sha256(&plat_buf);
        TPM_PCR_BANK[1].copy_from_slice(&plat_digest);

        let mut opt_buf = [0u8; 16];
        opt_buf[0..2].copy_from_slice(&0x8086u16.to_le_bytes());
        opt_buf[2..4].copy_from_slice(&hw.vendor_id.to_le_bytes());
        opt_buf[4..8].copy_from_slice(&0x0000_0000u32.to_le_bytes());
        let opt_digest = sha256(&opt_buf);
        TPM_PCR_BANK[2].copy_from_slice(&opt_digest);

        let sec_state = [
            0x01u8, 0x00, 0x00, 0x00, 0x53, 0x45, 0x43, 0x42, 0x53, 0x4D, 0x41, 0x50, 0x53, 0x4D,
            0x45, 0x50,
        ];
        let sec_digest = sha256(&sec_state);
        TPM_PCR_BANK[7].copy_from_slice(&sec_digest);

        log_event_internal(0, EV_POST_CODE, bios_digest, "BIOS_IVT_MEASURED");
        log_event_internal(1, EV_PLATFORM_CONFIG_FLAGS, plat_digest, "PLATFORM_CONFIG");
        log_event_internal(2, EV_ACTION, opt_digest, "HOST_BUS_TOPOLOGY");
        log_event_internal(7, EV_SEPARATOR, sec_digest, "SECUREBOOT_POLICY");

        TPM_INITIALIZED = true;
    }
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

    let data_hash = sha256(data);
    let mut extend_buf = [0u8; 64];
    unsafe {
        extend_buf[..32].copy_from_slice(&TPM_PCR_BANK[pcr_index]);
    }
    extend_buf[32..].copy_from_slice(&data_hash);
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

/// Measure genuine kernel text segment in physical RAM into PCR 4.
pub fn measure_kernel_code(code: &[u8]) -> Result<[u8; 32], &'static str> {
    extend_pcr(4, code, "KERNEL_TEXT_SEGMENT")
}

/// Measure Multiboot initrd ramdisk payload into PCR 5.
pub fn measure_initrd(initrd: &[u8]) -> Result<[u8; 32], &'static str> {
    extend_pcr(5, initrd, "INITRD_ARCHIVE")
}

/// Measure userland executable binary image prior to Ring 3 execution into PCR 10.
pub fn measure_binary(binary: &[u8], name: &str) -> Result<[u8; 32], &'static str> {
    extend_pcr(10, binary, name)
}

/// Internal helper to record an event in the ring log.
///
/// # Safety
///
/// Mutates global event log ring buffer static state without internal synchronization.
/// The caller must ensure serialized or single-threaded invocation.
pub(crate) unsafe fn log_event_internal(pcr: u8, ev_type: u32, digest: [u8; 32], desc: &str) {
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

/// Retrieve a slice of active measured boot events.
///
/// # Safety
/// The caller must ensure no concurrent writes to the event log occur during read.
pub unsafe fn get_event_log_slice() -> [Option<TpmEvent>; TPM_EVENT_LOG_CAPACITY] {
    TPM_EVENT_LOG
}
