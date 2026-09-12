; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; 64-Bit Long Mode Execution Trampoline (x86_64)

%include "constants.inc"

extern kernel_main
extern __bss_start
extern __bss_end
extern stack_top

section .text
bits 64
global _start64

_start64:
    ; Retrieve Multiboot2 information pointer pushed during 32-bit entry
    mov r12d, dword [rel stack_top - 4]

    ; Step 1: Reload segment registers with 64-bit data segment selector
    mov ax, GDT_DATA64_SEL
    mov ds, ax
    mov es, ax
    mov ss, ax
    xor ax, ax
    mov fs, ax
    mov gs, ax

    ; Step 2: Establish 64-bit kernel stack pointer
    mov rsp, stack_top

    ; Step 3: Zero-initialize kernel BSS section
    mov rdi, __bss_start
    mov rcx, __bss_end
    sub rcx, rdi
    xor al, al
    rep stosb

    ; Step 4: Transfer execution directly to 64-bit Rust kernel entry point
    mov rdi, r12
    call kernel_main

.hang:
    cli
    hlt
    jmp .hang
