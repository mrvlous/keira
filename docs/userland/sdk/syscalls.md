<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Ring 3 System Call Assembly Wrappers (`sys/syscall.h`)

This document specifies low-level inline assembly wrappers invoking kernel system calls from Userland C code.

---

## Dual-Architecture Register Mapping

| Argument | `x86_64` (SysV) | `i686` (cdecl / int 0x80) |
| :--- | :--- | :--- |
| Vector (`num`) | `RAX` | `EAX` |
| Argument 1 (`a1`) | `RDI` | `EBX` |
| Argument 2 (`a2`) | `RSI` | `ECX` |
| Argument 3 (`a3`) | `RDX` | `EDX` |
| Argument 4 (`a4`) | `R10` | `ESI` |
| Argument 5 (`a5`) | `R8` | `EDI` |
| Argument 6 (`a6`) | `R9` | `EBP` (via stack swap) |
| Return Value | `RAX` | `EAX` |

---

## 1. 64-bit Assembly Conventions (`x86_64`)

```c
int64_t syscall0(uint64_t num);
int64_t syscall1(uint64_t num, uint64_t a1);
int64_t syscall2(uint64_t num, uint64_t a1, uint64_t a2);
int64_t syscall3(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3);
int64_t syscall4(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4);
int64_t syscall5(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4, uint64_t a5);
int64_t syscall6(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3, uint64_t a4, uint64_t a5, uint64_t a6);
```

On `x86_64`, invoking `syscall6` utilizes `r10`, `r8`, and `r9` for upper parameters:

```c
int64_t syscall6(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3,
                 uint64_t a4, uint64_t a5, uint64_t a6) {
    int64_t ret;
    register uint64_t r10 __asm__("r10") = a4;
    register uint64_t r8  __asm__("r8")  = a5;
    register uint64_t r9  __asm__("r9")  = a6;
    __asm__ volatile("syscall"
                     : "=a"(ret)
                     : "a"(num), "D"(a1), "S"(a2), "d"(a3), "r"(r10), "r"(r8), "r"(r9)
                     : "rcx", "r11", "memory");
    return ret;
}
```

---

## 2. 32-bit Assembly Conventions (`i686`)

On `i686`, kernel system calls transition via software interrupt `int $0x80`. In `syscall6`, the 6th argument (`EBP`) is safely preserved and forwarded:

```c
int64_t syscall6(uint64_t num, uint64_t a1, uint64_t a2, uint64_t a3,
                 uint64_t a4, uint64_t a5, uint64_t a6) {
    int32_t ret;
    uint32_t arg6 = (uint32_t)a6;
    __asm__ volatile("pushl %%ebp\n\t"
                     "movl %7, %%ebp\n\t"
                     "int $0x80\n\t"
                     "popl %%ebp\n\t"
                     : "=a"(ret)
                     : "a"((uint32_t)num), "b"((uint32_t)a1), "c"((uint32_t)a2), "d"((uint32_t)a3),
                       "S"((uint32_t)a4), "D"((uint32_t)a5), "m"(arg6)
                     : "memory");
    return (int64_t)ret;
}
```
