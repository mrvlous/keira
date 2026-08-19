// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! Provides bare-metal, `no_std` cryptographic algorithms:
//!   - SHA-256 (FIPS 180-4) Message Digest & HMAC-SHA-256 (RFC 2104)
//!   - AES-128 Block Cipher & AES-128-GCM (NIST SP 800-38D) AEAD
//!   - Curve25519 (RFC 7748) Elliptic Curve Diffie-Hellman Key Exchange

pub mod aes;
pub mod curve25519;
pub mod sha256;

/// Hardware Security TPM 2.0 Enclave Subsystem
pub mod tpm;
