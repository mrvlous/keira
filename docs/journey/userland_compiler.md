<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Journey Milestone 6: Userland Runtime & Native C Compiler

The culmination of the Keira journey: running unprivileged Ring 3 applications and compiling C code natively inside the running kernel.

---

## Key Achievements

1. **Privilege Separation**: Switching to Ring 3 via `IRETQ` / `IRETD` with isolated user stacks and page tables.
2. **Syscall Boundary**: Register-based ABI (`syscall` on x86_64, `int 0x80` on i686) with safe user copying.
3. **Freestanding C SDK (`libc.a`)**: Comprehensive POSIX standard headers and runtime library.
4. **Native KCC Compiler**: In-kernel C compiler parsing C source code and generating freestanding Ring 3 ELF binaries.
