// SPDX-License-Identifier: GPL-2.0-only

//! Unit tests for AES-128-GCM authenticated encryption and authentication tag verification.

use super::*;

#[test]
fn test_aes128_gcm_roundtrip() {
    let key = [0x55u8; 16];
    let iv = [0xAAu8; 12];
    let plaintext = b"Keira Kernel Security Subsystem Payload 2026";
    let aad = b"AUTH_HEADER_METADATA";

    let mut ciphertext = [0u8; 64];
    let tag = aes128_gcm_encrypt(
        &key,
        &iv,
        plaintext,
        aad,
        &mut ciphertext[..plaintext.len()],
    );

    let mut decrypted = [0u8; 64];
    let ok = aes128_gcm_decrypt(
        &key,
        &iv,
        &ciphertext[..plaintext.len()],
        aad,
        &tag,
        &mut decrypted[..plaintext.len()],
    );

    assert!(ok, "Authentication tag verification must succeed");
    assert_eq!(&decrypted[..plaintext.len()], plaintext);

    // Verify tamper detection on ciphertext
    ciphertext[0] ^= 0x01;
    let corrupted = aes128_gcm_decrypt(
        &key,
        &iv,
        &ciphertext[..plaintext.len()],
        aad,
        &tag,
        &mut decrypted[..plaintext.len()],
    );
    assert!(!corrupted, "Corrupted ciphertext must fail tag validation");
}
