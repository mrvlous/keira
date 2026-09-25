<!-- SPDX-License-Identifier: GPL-2.0-only -->

# C Runtime Startup (`crt0.S`)

The C runtime initialization stub provides the entry point for all userland ELF executables executing in Ring 3.

---

## Process Startup Protocol

When the kernel loader (`crates/fs/src/elf/loader/`) parses an ELF binary and creates a new Ring 3 task:
1. The kernel pushes `argc`, `argv`, and `envp` onto the userland stack.
2. The instruction pointer is set to the ELF entry address (`_start`).
3. `crt0.S` retrieves arguments from the stack, aligns the stack frame, and calls `main(argc, argv, envp)`.
4. Upon return from `main()`, `crt0.S` invokes `exit(retval)` via the `SYS_exit` system call.

---

## i686 Implementation (`userland/arch/x86/i686/crt0.S`)

```assembly
.global _start
.extern main
.extern exit

.section .text
_start:
    xorl %ebp, %ebp
    popl %eax              # argc
    movl %esp, %edx        # argv
    pushl %edx
    pushl %eax
    call main
    addl $8, %esp
    pushl %eax             # exit code
    call exit
1:  hlt
    jmp 1b
```

---

## x86_64 Implementation (`userland/arch/x86/x86_64/crt0.S`)

```assembly
.global _start
.extern main
.extern exit

.section .text
_start:
    xorq %rbp, %rbp
    popq %rdi              # argc -> 1st argument (System V AMD64 ABI)
    movq %rsp, %rsi        # argv -> 2nd argument
    andq $-16, %rsp        # 16-byte stack alignment
    call main
    movq %rax, %rdi        # exit code -> 1st argument
    call exit
1:  hlt
    jmp 1b
```
