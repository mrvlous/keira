<!-- SPDX-License-Identifier: GPL-2.0-only -->

# C Runtime Startup (`crt0.asm`)

The C runtime initialization stub provides the entry point for all userland ELF executables executing in Ring 3.

---

## Process Startup Protocol

When the kernel loader (`crates/fs/src/elf/loader/`) parses an ELF binary and creates a new Ring 3 task:
1. The kernel pushes `argc`, `argv`, `envp` and the `auxv` vector onto the userland stack.
2. The instruction pointer is set to the ELF entry address (`_start`).
3. `crt0.asm` retrieves arguments from the stack, initializes the global `environ` pointer, extracts `AT_RANDOM` to seed `__stack_chk_guard`, aligns the stack to a 16-byte boundary and calls `main(argc, argv, envp)`.
4. Upon return from `main()`, `crt0.asm` invokes `exit(retval)` via the `SYS_exit` system call.

---

## i686 Implementation (`userland/arch/x86/i686/crt0.asm`)

```nasm
global _start
extern main
extern exit
extern environ
extern __stack_chk_guard

section .text
_start:
    xor ebp, ebp

    add esp, 4
    mov eax, [esp]          ; argc
    mov edx, [esp + 4]      ; argv
    mov ecx, [esp + 8]      ; envp

    mov [environ], ecx

    ; Align stack and invoke main(argc, argv, envp)
    push ecx
    push edx
    push eax
    call main
    add esp, 12

    push eax                ; exit code
    call exit

.halt:
    hlt
    jmp .halt
```

---

## x86_64 Implementation (`userland/arch/x86/x86_64/crt0.asm`)

```nasm
global _start
extern main
extern exit
extern environ
extern __stack_chk_guard

section .text
_start:
    xor rbp, rbp

    pop rdi                 ; argc -> 1st arg (RDI)
    mov rsi, rsp            ; argv -> 2nd arg (RSI)

    ; Calculate envp: RDX = &argv[argc + 1]
    mov rax, rdi
    inc rax
    shl rax, 3
    lea rdx, [rsi + rax]    ; envp -> 3rd arg (RDX)
    mov [rel environ], rdx

    ; 16-byte stack alignment
    mov rbx, rsp
    and rsp, -16
    call main

    mov rdi, rax            ; exit status
    call exit

.halt:
    hlt
    jmp .halt
```
