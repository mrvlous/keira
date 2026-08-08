<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Bare-Metal Cryptographic Subsystem (`kernel/src/crypto/`)

This document details the architecture, mathematical algorithms, and Rust `no_std` implementation of the core Cryptographic Subsystem in Keira Kernel.

---

## 1. Overview

The cryptographic subsystem ([kernel/src/crypto/](../../kernel/src/crypto/)) provides bare-metal, dependency-free cryptographic primitives to kernel space modules (such as the Native TLS 1.3 Engine in `net/tls.rs`):

* **SHA-256 & HMAC-SHA-256 ([sha256.rs](../../kernel/src/crypto/sha256.rs))**: Message digest hashing (FIPS 180-4) and keyed-hash message authentication (RFC 2104).
* **AES-128 & AES-128-GCM ([aes.rs](../../kernel/src/crypto/aes.rs))**: Symmetric block cipher and Galois/Counter Mode authenticated encryption with associated data (NIST SP 800-38D).
* **Curve25519 X25519 ([curve25519.rs](../../kernel/src/crypto/curve25519.rs))**: Elliptic Curve Diffie-Hellman (ECDH) key exchange using Montgomery ladder scalar multiplication (RFC 7748).

---

## 2. SHA-256 Message Digest (FIPS 180-4)

The SHA-256 engine computes 256-bit (32-byte) message digests:
1. **Initial Hash Values**: Standard 8 × 32-bit fractional parts of square roots of prime numbers (`0x6a09e667` ...).
2. **Compression Loop**: Processes 512-bit (64-byte) data blocks through 64 rounds using bitwise rotation constants (`SHA256_K`).
3. **HMAC-SHA-256**: Generates 256-bit authentication codes via inner (`0x36`) and outer (`0x5c`) key padding.

---

## 3. AES-128-GCM Authenticated Encryption (NIST SP 800-38D)

* **Rijndael Block Cipher**: Implements 10 rounds of `SubBytes` (S-Box lookup), `ShiftRows`, `MixColumns` (GF(2^8) Galois Field arithmetic `gf_mul2`/`gf_mul3`), and `AddRoundKey`.
* **Galois/Counter Mode (GCM)**: Performs CTR mode encryption combined with GHASH authentication over Galois Field GF(2^128) using 16-byte multiplication.

---

## 4. Curve25519 X25519 Key Exchange (RFC 7748)

* **Scalar Multiplication**: Computes $Q = s \cdot P$ over Curve25519 ($y^2 = x^3 + 486662x^2 + x \pmod{2^{255}-19}$).
* **Limbed Field Arithmetic**: Represents field elements using 5 × 51-bit limbs (`u64`) to prevent integer overflow.
* **Security Properties**: Uses constant-time conditional swap (`cswap`) to protect against side-channel timing attacks.
