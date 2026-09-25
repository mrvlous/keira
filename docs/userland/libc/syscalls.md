<!-- SPDX-License-Identifier: GPL-2.0-only -->

# System Call Trapping Interface (`userland/lib/syscall/`)

The syscall wrapper module bridges standard C function calls to kernel system call dispatchers via architecture-specific trap instructions.

---

## i686 Syscall Trap (`int 0x80`)

Arguments passed via registers: `%eax` (syscall number), `%ebx` (arg1), `%ecx` (arg2), `%edx` (arg3), `%esi` (arg4), `%edi` (arg5).

```c
long syscall3(long num, long a1, long a2, long a3) {
    long ret;
    asm volatile (
        "int $0x80"
        : "=a"(ret)
        : "a"(num), "b"(a1), "c"(a2), "d"(a3)
        : "memory"
    );
    return ret;
}
```

---

## x86_64 Syscall Trap (`syscall`)

Arguments passed via registers: `%rax` (syscall number), `%rdi` (arg1), `%rsi` (arg2), `%rdx` (arg3), `%r10` (arg4), `%r8` (arg5), `%r9` (arg6).

```c
long syscall3(long num, long a1, long a2, long a3) {
    long ret;
    asm volatile (
        "syscall"
        : "=a"(ret)
        : "a"(num), "D"(a1), "S"(a2), "d"(a3)
        : "rcx", "r11", "memory"
    );
    return ret;
}
```
