<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Native Transport Layer Security (TLS 1.3)

This document details bare-metal TLS 1.3 cryptographic handshaking and record encryption in Keira Kernel.

---

## Supported Cryptographic Suites

* **Key Exchange**: X25519 Elliptic Curve Diffie-Hellman (ECDHE).
* **Symmetric Cipher**: AES-128-GCM (Authenticated Encryption with Associated Data).
* **Hash & Key Derivation**: SHA-256 and HKDF (HMAC-based Key Derivation Function).

---

## Core API (`crates/net/src/tls/mod.rs`)

```rust
pub fn tls_connect(host: &str, ip: Ipv4Address, port: u16) -> Result<TlsStream, &'static str>;
pub fn tls_send(stream: &mut TlsStream, data: &[u8]) -> Result<usize, &'static str>;
pub fn tls_recv(stream: &mut TlsStream, buf: &mut [u8]) -> Result<usize, &'static str>;
```
