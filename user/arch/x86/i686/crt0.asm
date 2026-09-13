; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Standard C Runtime 0 (crt0) for i686 Userland
;
; Initializes the 32-bit x86 CDECL userland environment, extracts
; argc, argv, and envp from the initial user stack, aligns the stack to
; a 16-byte boundary, executes main(), and automatically exits with the
; return code via the exit() standard library function.

global _start
extern main
extern exit
extern environ

section .text
_start:
    ; Terminate stack frame unwinding by clearing EBP
    xor ebp, ebp

    ; Kernel stack setup in run.rs has [ret_sentinel, argc, argv, envp]
    ; Advance past sentinel to access parameters
    add esp, 4
    mov eax, [esp]          ; argc
    mov edx, [esp + 4]      ; argv
    mov ecx, [esp + 8]      ; envp

    ; Store envp in global environ pointer
    mov [environ], ecx

    ; Align stack pointer to 16 bytes
    and esp, -16

    ; Push arguments for main(argc, argv, envp) with 4-byte pad for CDECL call alignment
    sub esp, 4
    push ecx                ; envp
    push edx                ; argv
    push eax                ; argc

    ; Call standard C application entrypoint: main(argc, argv, envp)
    call main

    ; Pass return value of main in EAX to exit()
    push eax
    call exit

.hang:
    pause
    jmp .hang
