<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Trusted Platform Module (TPM 2.0) Enclave

This document specifies hardware TPM 2.0 communication over memory-mapped I/O in Keira Kernel.

---

## Memory-Mapped Registers (CRB Interface)

* **Base Address**: Physical `0xFED40000`.
* **Locality 0**: Control (`0x00`), Status (`0x18`), Command Buffer (`0x80`), Response Buffer (`0x80`).

---

## Core API (`crates/crypto/src/tpm.rs`)

```rust
pub unsafe fn tpm_init() -> Result<(), &'static str>;
pub unsafe fn tpm_get_random(out_buf: &mut [u8]) -> Result<(), &'static str>;
pub unsafe fn tpm_pcr_read(pcr_idx: u32, out_hash: &mut [u8; 32]) -> Result<(), &'static str>;
```
