; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; 32-Bit Protected Mode Bootstrap Entry Point (i686)
;
; Initial entry point executed after Multiboot2 bootloader transfers control.
; Running in 32-bit protected mode.

%include "constants.inc"

section .bss
align 16

stack_bottom:
    resb KERNEL_STACK_SIZE
global stack_top
stack_top:

section .text
bits 32
global _start

extern kernel_main
extern gdt_descriptor

_start:
    ; Step 1: Initialize temporary bootstrap stack pointer
    mov esp, stack_top

    ; Step 2: Validate Multiboot2 magic signature in EAX
    cmp eax, MULTIBOOT2_BOOTLOADER
    jne .halt_no_multiboot

    ; Step 3: Load 32-bit Global Descriptor Table (GDT)
    lgdt [gdt_descriptor]
    jmp 0x08:.reload_cs

.reload_cs:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    ; Step 4: Pass Multiboot2 info pointer in EBX and jump to Rust kernel
    push ebx
    call kernel_main

.halt:
    cli
    hlt
    jmp .halt

.halt_no_multiboot:
    mov dword [VGA_BUFFER_ADDR], 0x4F4D
    jmp .halt
