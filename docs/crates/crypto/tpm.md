<!-- SPDX-License-Identifier: GPL-2.0-only -->

# TPM 2.0 Hardware Security Enclave

Documentation for TPM driver in [`crates/crypto/src/tpm/tpm2.rs`](../../../crates/crypto/src/tpm/tpm2.rs).

## Hardware Interface
- Communicates with on-board TPM 2.0 chips via LPC/MMIO register space at physical base `0xFED40000`.
- Supports hardware Random Number Generation (`TPM2_GetRandom`), PCR register reading (`TPM2_PCR_Read`), and cryptographic key sealing.
- System Call: `sys_tpm_quote` (Syscall 43).
