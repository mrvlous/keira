<!-- SPDX-License-Identifier: GPL-2.0-only -->

# AES-128 & AES-128-GCM

Documentation for AES block cipher and Galois/Counter Mode in [`crates/crypto/src/cipher/aes.rs`](../../../crates/crypto/src/cipher/aes.rs).

## Specifications
- **AES-128**: 10-round substitution-permutation network operating on 128-bit blocks with Rijndael S-box substitution, ShiftRows, MixColumns, and AddRoundKey.
- **AES-128-GCM (NIST SP 800-38D)**: Combines AES counter mode (CTR) encryption with Galois field ($	ext{GF}(2^{128})$) GHASH authentication to produce a 16-byte authentication tag for AEAD ciphers.
