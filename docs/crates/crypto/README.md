<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `keira-crypto` - Bare-Metal Cryptography

The `keira-crypto` crate implements dependency-free cryptographic hashing, symmetric encryption, authenticated ciphers, elliptic curve key exchange, and hardware security enclave drivers.

## Submodules

- [`sha256.md`](sha256.md): FIPS 180-4 SHA-256 and RFC 2104 HMAC-SHA-256.
- [`aes.md`](aes.md): AES-128 block cipher and NIST SP 800-38D AES-128-GCM.
- [`curve25519.md`](curve25519.md): RFC 7748 Montgomery ladder X25519 ECDH.
- [`tpm.md`](tpm.md): TPM 2.0 LPC/MMIO hardware security enclave.
