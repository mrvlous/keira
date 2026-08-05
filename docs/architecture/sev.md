# AMD SEV & Intel TDX Confidential Computing Subsystem

This document details hardware Secure Encrypted Virtualization (SEV-SNP/TDX) memory page isolation in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel implements hardware memory encryption ([sev.rs](../../kernel/src/arch/sev.rs), **Syscall 61 `sys_sev`**) supporting AMD SEV-SNP and Intel TDX confidential computing enclaves.

---

## 2. System Call Interface

```c
// Syscall 61: Query/Activate confidential memory encryption enclave
long sys_sev(unsigned int cmd, uint64_t page_addr);
```

---

## 3. Kernel APIs

*   `pub fn sys_sev(cmd: u32, page_addr: u64) -> Result<u64, &'static str>`: Issues hardware SEV/TDX MSR instructions for page state validation.
