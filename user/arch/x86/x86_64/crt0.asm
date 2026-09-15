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
extern environ
extern __stack_chk_guard

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

    ; Store envp in global environ pointer
    mov [rel environ], rdx

    ; Scan past envp to locate auxv table
    mov rcx, rdx
.find_auxv:
    cmp qword [rcx], 0
    lea rcx, [rcx + 8]
    jne .find_auxv

    ; Scan auxv pairs for AT_RANDOM (25)
.scan_auxv:
    mov r8, [rcx]
    test r8, r8
    jz .auxv_done
    cmp r8, 25
    jne .next_auxv
    mov r9, [rcx + 8]
    test r9, r9
    jz .auxv_done
    mov r10, [r9]
    test r10, r10
    jz .auxv_done
    mov [rel __stack_chk_guard], r10
    jmp .auxv_done
.next_auxv:
    add rcx, 16
    jmp .scan_auxv
.auxv_done:

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
