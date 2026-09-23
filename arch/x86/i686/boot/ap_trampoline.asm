; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Application Processor (AP) 16-Bit Real-Mode Bootstrap Trampoline (i686)

%include "constants.inc"

%define AP_TRAMPOLINE_BASE 0x8000

section .rodata
global ap_trampoline_start
global ap_trampoline_end
global ap_cr3_val
global ap_stack_val
global ap_entry_val
global ap_core_id
global ap_status_flag

ap_trampoline_start:
[bits 16]
ap_real16_entry:
    cli
    cld
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00

    ; Load temporary 32-bit GDT descriptor
    lgdt [AP_TRAMPOLINE_BASE + (ap_gdt_desc - ap_trampoline_start)]

    ; Activate Protected Mode (CR0.PE = 1)
    mov eax, cr0
    or al, 1
    mov cr0, eax

    ; Far jump to 32-bit Protected Mode code
    jmp dword 0x08:(AP_TRAMPOLINE_BASE + (ap_pm32_entry - ap_trampoline_start))

[bits 32]
ap_pm32_entry:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax

    ; Load assigned kernel stack pointer for this AP core
    mov esp, [AP_TRAMPOLINE_BASE + (ap_stack_val - ap_trampoline_start)]

    ; Push core_id argument for cdecl calling convention: ap_main(core_id: usize)
    push dword [AP_TRAMPOLINE_BASE + (ap_core_id - ap_trampoline_start)]

    ; Transfer control to Rust AP initialization entrypoint
    mov eax, [AP_TRAMPOLINE_BASE + (ap_entry_val - ap_trampoline_start)]
    call eax

.ap_halt:
    cli
    hlt
    jmp .ap_halt

; Trampoline Parameter Block
align 4
ap_cr3_val:     dd 0
ap_stack_val:   dd 0
ap_entry_val:   dd 0
ap_core_id:     dd 0
ap_status_flag: dd 0

; 32-bit GDT
align 16
ap_gdt_start:
    dq 0x0000000000000000
    dq 0x00CF9A000000FFFF
    dq 0x00CF92000000FFFF
ap_gdt_end:

ap_gdt_desc:
    dw ap_gdt_end - ap_gdt_start - 1
    dd AP_TRAMPOLINE_BASE + (ap_gdt_start - ap_trampoline_start)

align 16
ap_trampoline_end:
