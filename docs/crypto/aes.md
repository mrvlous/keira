<!-- SPDX-License-Identifier: GPL-2.0-only -->

# AES & AES-GCM Block Ciphers

This document details AES-128 and authenticated AES-128-GCM encryption in Keira Kernel.

---

## Technical Specifications

* **Key Size**: 128 bits (16 bytes).
* **Block Size**: 128 bits (16 bytes).
* **Authenticated Mode**: Galois/Counter Mode (GCM) with 16-byte GHASH tag authentication.

---

## Core API (`crates/crypto/src/aes.rs`)

```rust
pub fn aes128_encrypt_block(key: &[u8; 16], block: &mut [u8; 16]);
pub fn aes128_gcm_encrypt(key: &[u8; 16], iv: &[u8; 12], aad: &[u8], plaintext: &[u8], ciphertext: &mut [u8], tag: &mut [u8; 16]);
pub fn aes128_gcm_decrypt(key: &[u8; 16], iv: &[u8; 12], aad: &[u8], ciphertext: &[u8], tag: &[u8; 16], plaintext: &mut [u8]) -> Result<(), &'static str>;
```
