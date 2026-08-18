<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Hardware Security Enclave & TPM 2.0 Subsystem

This document details the Trusted Platform Module (TPM 2.0) MMIO interface, Platform Configuration Register (PCR) measurement banks, and hardware cryptographic key storage in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel integrates a hardware security enclave driver ([tpm.rs](../../kernel/src/crypto/tpm.rs)) interfacing with on-board TPM 2.0 chips via LPC or Memory-Mapped I/O (MMIO) register space at physical address `0xFED40000`.

---

## 2. PCR Measurement Banks

The TPM maintains 24 Platform Configuration Registers (PCR 0..23) storing SHA-256 cryptographic hashes representing the measured state of system boot components:

| PCR Index | Target Measurement | Description |
| :---: | :--- | :--- |
| `PCR 0` | Firmware & BIOS | Core motherboard firmware code and configuration |
| `PCR 4` | GRUB Bootloader | Bootloader executable code and kernel command line |
| `PCR 8` | Keira Kernel Code | SHA-256 hash measurement of 64-bit kernel image |
| `PCR 9` | Initrd Boot Archive | USTAR Tar initrd RAM disk integrity hash |

---

## 3. Kernel APIs

*   `pub fn init()`: Maps TPM MMIO register space at `0xFED40000` and validates TPM 2.0 command readiness.
*   `pub fn read_pcr(pcr_index: u32) -> Result<[u8; 32], &'static str>`: Reads the 256-bit SHA-256 digest from the requested PCR bank.
