<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel Security Architecture

Keira Kernel employs a defense-in-depth security model:

1. **Mandatory Access Control (MAC)**: Path-based access rules restrict userland programs from modifying critical kernel configuration files.
2. **Secure Computing (Seccomp)**: BPF bytecode filter evaluated on every system call to whitelist only authorized syscall numbers.
3. **No-Execute (NX / XD Bit)**: Configured in paging page table entries to prevent execution of userland stack or heap memory.
4. **Hardware Security Enclave (TPM 2.0)**: Hardware-rooted platform measurement and PCR quote validation.
