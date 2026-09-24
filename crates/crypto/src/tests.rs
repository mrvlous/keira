// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Integration tests for TPM 2.0 measured boot and sealed storage facilities.

use super::*;

#[test]
fn test_tpm2_pcr_init_and_extend() {
    tpm::init();
    assert!(tpm::is_initialized());

    let pcr0 = tpm::read_pcr(0).expect("PCR 0 should be readable");
    assert_ne!(pcr0, [0u8; 32]);

    let extended =
        tpm::extend_pcr(10, b"measurement_payload", "TEST_MEASURE").expect("Extend should succeed");
    assert_ne!(extended, [0u8; 32]);

    let pcr10 = tpm::read_pcr(10).expect("PCR 10 should be readable");
    assert_eq!(extended, pcr10);

    let quote = tpm::quote_pcrs(0b0001);
    assert_ne!(quote, [0u8; 32]);
}

#[test]
fn test_tpm2_sealed_storage_roundtrip() {
    tpm::init();
    let secret = b"TOP_SECRET_DISK_ENCRYPTION_KEY_128BIT";
    let mask = 1u32 << 16;

    let blob = tpm::seal_secret(secret, mask).expect("Sealing should succeed");
    assert_eq!(blob.magic, tpm::TPM_SEALED_MAGIC);
    assert_eq!(blob.pcr_mask, mask);
    assert_eq!(blob.data_len as usize, secret.len());

    let mut out_buf = [0u8; 64];
    let unsealed_len = tpm::unseal_secret(&blob, &mut out_buf).expect("Unsealing should succeed");
    assert_eq!(unsealed_len, secret.len());
    assert_eq!(&out_buf[..unsealed_len], secret);
}

#[test]
fn test_tpm2_sealed_storage_tamper_detection() {
    tpm::init();
    let secret = b"PASSCODE_FOR_SENSITIVE_SERVICE";
    let mask = 1u32 << 17;

    let blob = tpm::seal_secret(secret, mask).expect("Sealing should succeed");

    let _ = tpm::extend_pcr(17, b"malicious_kernel_payload", "ATTACK_TAMPER");

    let mut out_buf = [0u8; 64];
    let result = tpm::unseal_secret(&blob, &mut out_buf);
    assert!(
        result.is_err(),
        "Unseal must fail when PCR state does not match sealed quote"
    );
}

#[test]
fn test_tpm2_measured_boot_primitives() {
    tpm::init();
    let fake_kernel_text = [0x90u8; 128];
    let res_code = tpm::measure_kernel_code(&fake_kernel_text);
    assert!(res_code.is_ok());

    let fake_initrd = [0x55u8; 256];
    let res_initrd = tpm::measure_initrd(&fake_initrd);
    assert!(res_initrd.is_ok());

    let fake_elf = [0x7Fu8, b'E', b'L', b'F'];
    let res_bin = tpm::measure_binary(&fake_elf, "sysinfo.elf");
    assert!(res_bin.is_ok());
}
