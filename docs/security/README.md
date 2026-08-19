<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Hardware & Software Security Subsystems

Welcome to the Security documentation section for Keira Kernel.

## Documents

* [Cryptographic Subsystem](crypto.md): Native SHA-256, HMAC, AES-128-GCM AEAD, and Curve25519 X25519 ECDH key exchange.
* [Hardware Security TPM 2.0 Enclave](tpm.md): Trusted Platform Module MMIO interface, PCR measurement banks, and hardware key storage.
* [NX Bit & KASLR Hardware Security](nx.md): Hardware No-Execute (NX) page protection and KASLR randomization.
* [Mandatory Access Control (MAC)](mac.md): Path-based security rule evaluation and process sandboxing policies.
* [Seccomp BPF Syscall Filtering Sandbox](seccomp.md): In-kernel BPF system call sandbox filtering (`sys_seccomp`).
