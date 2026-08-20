<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Native TLS 1.3 Cryptographic Engine

Documentation for native TLS 1.3 in [`crates/net/src/tls/`](../../../crates/net/src/tls).

## Architecture
- Completely self-contained without external dependencies.
- **Key Exchange**: X25519 Elliptic Curve Diffie-Hellman (`supported_groups: 0x001d`).
- **Key Schedule**: HKDF-Extract and HKDF-Expand-Label (SHA-256) deriving client/server handshake and application traffic keys.
- **Record Layer**: Encrypted AEAD records using `TLS_AES_128_GCM_SHA256`.
