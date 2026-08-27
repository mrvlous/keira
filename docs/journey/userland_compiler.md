<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Milestone 6: Ring 3 Privilege Separation & Native C Compiler

This journal entry details the transition to userland execution, Task State Segment stack switching, system call validation, and compiling C programs inside Keira Kernel.

---

## Engineering Challenges

1. **Privilege Boundary Security**: Userland processes run in unprivileged CPU Ring 3 and may pass invalid, malicious, or unmapped memory pointers to the kernel.
2. **Dynamic ELF Loading**: Parsing 64-bit ELF headers, verifying magic bytes, creating private virtual address mappings, and mapping `PT_LOAD` segments with proper `R/W/X` permissions.
3. **Native In-Kernel Toolchains**: Running a real C compiler (`kcc.elf`) directly within Keira to compile user programs on bare metal.

---

## Solutions & Design Choices

* **Hardened User Pointer Validation**: Built `validate_user_ptr()` and `copy_from_user()` verifying non-nullness, canonical address bounds, and overflow prevention before any pointer dereference.
* **Integrated KCC Compiler**: Integrated a freestanding C compiler (`kcc.elf`) into `/system/bin/`, allowing developers to write, compile, and execute C programs directly within Keira.
