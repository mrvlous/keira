; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Standard C Runtime 0 (crt0) for x86_64 Userland
;
; Initializes the System V AMD64 ABI userland environment, extracts
; argc, argv, and envp from the initial user stack, aligns the stack to
; a 16-byte boundary, executes main(), and automatically exits with the
; return code via the exit() standard library function.

global _start
extern main
extern exit

section .text
_start:
    ; Terminate stack frame unwinding by clearing RBP
    xor rbp, rbp

    ; Extract argc from top of stack
    pop rdi

    ; RSI points to argv[0] on the stack
    mov rsi, rsp

    ; Compute envp: rdx = &argv[argc + 1]
    mov rax, rdi
    inc rax
    shl rax, 3
    lea rdx, [rsi + rax]

    ; Align stack pointer to 16 bytes before calling C code
    and rsp, -16

    ; Call standard C application entrypoint: main(argc, argv, envp)
    call main

    ; Pass return value of main in RDI to exit()
    mov rdi, rax
    call exit

.hang:
    pause
    jmp .hang
