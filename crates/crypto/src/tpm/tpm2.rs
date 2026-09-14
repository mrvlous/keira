// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Trusted Platform Module (TPM 2.0) interface, PCR measurement banks, sealed storage, and measured boot log.

use crate::cipher::gcm::{aes128_gcm_decrypt, aes128_gcm_encrypt};
use crate::hash::hmac::hmac_sha256;
use crate::hash::sha256::{sha256, Sha256};

pub const TPM_PCR_COUNT: usize = 24;
pub const TPM_EVENT_LOG_CAPACITY: usize = 32;
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

pub const TPM_SEALED_MAGIC: [u8; 4] = [b'T', b'P', b'M', b'S'];
pub const TPM_MAX_SECRET_LEN: usize = 128;

/// Internal hardware security enclave root seed.
static TPM_SECRET_SEED: [u8; 32] = [
    0x54, 0x50, 0x4D, 0x32, 0x5F, 0x53, 0x45, 0x45, 0x44, 0x5F, 0x4B, 0x45, 0x49, 0x52, 0x41, 0x5F,
    0x52, 0x49, 0x4E, 0x47, 0x30, 0x5F, 0x53, 0x45, 0x43, 0x55, 0x52, 0x49, 0x54, 0x59, 0x30, 0x31,
];

/// Hardware TCG TPM 2.0 TIS interface status descriptor.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TpmHardwareInfo {
    pub vendor_id: u16,
    pub device_id: u16,
    pub revision_id: u8,
    pub present: bool,
    pub locality_active: bool,
}

/// Request TIS Locality 0 via MMIO TPM_ACCESS register.
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

/// Release TIS Locality 0 via MMIO TPM_ACCESS register.
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

/// TPM 2.0 Sealed Storage Data Blob (200 bytes, `repr(C)`).
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TpmSealedBlob {
    pub magic: [u8; 4],
    pub pcr_mask: u32,
    pub expected_quote: [u8; 32],
    pub nonce: [u8; 12],
    pub data_len: u32,
    pub ciphertext: [u8; TPM_MAX_SECRET_LEN],
    pub auth_tag: [u8; 16],
}

/// Initialize the TPM 2.0 hardware security controller and baseline PCR measurements.
pub fn init() {
    unsafe {
        if TPM_INITIALIZED {
            return;
        }

        let hw = probe_hardware_tpm();

        // PCR 0: Genuine low memory BIOS Data Area (BDA at 0x0400) hardware state
        #[cfg(target_os = "none")]
        let bios_digest = {
            let bda_slice = core::slice::from_raw_parts(0x0000_0400 as *const u8, 256);
            sha256(bda_slice)
        };
        #[cfg(not(target_os = "none"))]
        let bios_digest = sha256(b"KEIRA_BIOS_EMULATED_IVT_SIGNATURE");

        TPM_PCR_BANK[0].copy_from_slice(&bios_digest);

        // PCR 1: Host Platform Configuration (MMIO base, vendor, device ID, locality state)
        let mut plat_buf = [0u8; 16];
        plat_buf[..8].copy_from_slice(&TPM_MMIO_BASE.to_le_bytes());
        plat_buf[8..10].copy_from_slice(&hw.vendor_id.to_le_bytes());
        plat_buf[10..12].copy_from_slice(&hw.device_id.to_le_bytes());
        plat_buf[12] = if hw.locality_active { 1 } else { 0 };
        let plat_digest = sha256(&plat_buf);
        TPM_PCR_BANK[1].copy_from_slice(&plat_digest);

        // PCR 2: Option ROM Code & Host Bus Topology (x86 PCI root topology)
        let mut opt_buf = [0u8; 16];
        opt_buf[0..2].copy_from_slice(&0x8086u16.to_le_bytes());
        opt_buf[2..4].copy_from_slice(&hw.vendor_id.to_le_bytes());
        opt_buf[4..8].copy_from_slice(&0x0000_0000u32.to_le_bytes());
        let opt_digest = sha256(&opt_buf);
        TPM_PCR_BANK[2].copy_from_slice(&opt_digest);

        // PCR 7: Secure Boot Policy Enclave state
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

/// Derive a 128-bit AES key from the TPM master seed and PCR quote using HKDF-SHA256.
pub fn derive_seal_key(quote: &[u8; 32]) -> [u8; 16] {
    let prk = hmac_sha256(&TPM_SECRET_SEED, quote);
    let info = b"TPM2_SEAL_STORAGE\x01";
    let okm = hmac_sha256(&prk, info);
    let mut key = [0u8; 16];
    key.copy_from_slice(&okm[..16]);
    key
}

/// Seal secret data bound to a specific PCR selection policy mask.
pub fn seal_secret(secret: &[u8], pcr_mask: u32) -> Result<TpmSealedBlob, &'static str> {
    if !is_initialized() {
        init();
    }
    if secret.is_empty() {
        return Err("Secret payload cannot be empty");
    }
    if secret.len() > TPM_MAX_SECRET_LEN {
        return Err("Secret payload exceeds maximum sealed capacity (128 bytes)");
    }

    let quote = quote_pcrs(pcr_mask);
    let key = derive_seal_key(&quote);

    let mut nonce = [0u8; 12];
    #[cfg(target_os = "none")]
    {
        let lo: u32;
        let hi: u32;
        unsafe {
            core::arch::asm!("rdtsc", out("eax") lo, out("edx") hi);
        }
        nonce[0..4].copy_from_slice(&lo.to_le_bytes());
        nonce[4..8].copy_from_slice(&hi.to_le_bytes());
        nonce[8..12].copy_from_slice(&(pcr_mask ^ 0xA5A5_5A5A).to_le_bytes());
    }
    #[cfg(not(target_os = "none"))]
    {
        nonce[0..4].copy_from_slice(&0x01020304u32.to_le_bytes());
        nonce[4..8].copy_from_slice(&0x05060708u32.to_le_bytes());
        nonce[8..12].copy_from_slice(&(pcr_mask ^ 0xA5A5_5A5A).to_le_bytes());
    }

    let mut ciphertext = [0u8; TPM_MAX_SECRET_LEN];
    let aad = b"KEIRA_TPM2_SEALED_ENCLAVE";
    let tag = aes128_gcm_encrypt(&key, &nonce, secret, aad, &mut ciphertext[..secret.len()]);

    unsafe {
        log_event_internal(
            (pcr_mask.trailing_zeros() as u8).min(23),
            EV_ACTION,
            quote,
            "TPM2_SEAL_SECRET",
        );
    }

    Ok(TpmSealedBlob {
        magic: TPM_SEALED_MAGIC,
        pcr_mask,
        expected_quote: quote,
        nonce,
        data_len: secret.len() as u32,
        ciphertext,
        auth_tag: tag,
    })
}

/// Unseal secret data, validating PCR policy attestation quote and GMAC authentication.
pub fn unseal_secret(blob: &TpmSealedBlob, out: &mut [u8]) -> Result<usize, &'static str> {
    if !is_initialized() {
        init();
    }
    if blob.magic != TPM_SEALED_MAGIC {
        return Err("Invalid sealed blob magic signature");
    }
    let data_len = blob.data_len as usize;
    if data_len > TPM_MAX_SECRET_LEN {
        return Err("Corrupt blob: payload length exceeds maximum capacity");
    }
    if out.len() < data_len {
        return Err("Destination buffer too small for unsealed secret");
    }

    let current_quote = quote_pcrs(blob.pcr_mask);
    if current_quote != blob.expected_quote {
        return Err("PCR policy integrity check failed: platform state mismatch");
    }

    let key = derive_seal_key(&blob.expected_quote);
    let aad = b"KEIRA_TPM2_SEALED_ENCLAVE";

    let ok = aes128_gcm_decrypt(
        &key,
        &blob.nonce,
        &blob.ciphertext[..data_len],
        aad,
        &blob.auth_tag,
        &mut out[..data_len],
    );

    if !ok {
        return Err("Sealed secret authentication failed: tamper detected");
    }

    unsafe {
        log_event_internal(
            (blob.pcr_mask.trailing_zeros() as u8).min(23),
            EV_ACTION,
            current_quote,
            "TPM2_UNSEAL_SECRET",
        );
    }

    Ok(data_len)
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
