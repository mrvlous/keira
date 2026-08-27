<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Keira Kernel Cryptographic & Security Subsystems

The `crypto` subsystem provides bare-metal cryptographic primitives, hardware TPM 2.0 enclave communication, Seccomp BPF system call filters, and Mandatory Access Control (MAC).

---

## Cryptography & Security Index

| Component | Document | Description |
| :--- | :--- | :--- |
| **SHA-256 & HMAC** | [`sha256.md`](sha256.md) | Bare-metal SHA-256 hash function and HMAC key derivation |
| **AES & AES-GCM** | [`aes.md`](aes.md) | AES-128 block cipher and Galois/Counter Mode (GCM) AEAD encryption |
| **Curve25519** | [`curve25519.md`](curve25519.md) | X25519 Elliptic Curve Diffie-Hellman key agreement protocol |
| **TPM 2.0 Enclave** | [`tpm.md`](tpm.md) | Hardware Trusted Platform Module 2.0 commands and PCR measurement |
| **Seccomp BPF** | [`seccomp.md`](seccomp.md) | Task-level BPF system call sandboxing engine |
| **MAC Security** | [`mac.md`](mac.md) | Mandatory Access Control path and inode security policy enforcement |
