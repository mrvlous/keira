.. SPDX-License-Identifier: GPL-2.0-only


Native TLS 1.3 Encryption Engine
================================

Pure Rust RFC 8446 TLS 1.3 engine (``crates/net/src/tls/``) supporting X25519 ECDHE, AES-128-GCM, HKDF-SHA256 key schedule, DER X.509 SAN/CN verification (or ``TLS_FLAG_INSECURE``) and transparent TCP socket upgrade via ``SYS_TLS_CONNECT``.
