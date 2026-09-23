; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Application Processor (AP) 16-Bit Real-Mode Bootstrap Trampoline (x86_64)

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
    mov esp, 0x7C00

    ; Load CR3 with PML4 page table root passed by BSP
    mov eax, [AP_TRAMPOLINE_BASE + (ap_cr3_val - ap_trampoline_start)]
    mov cr3, eax

    ; Enable Physical Address Extension (PAE) in CR4
    mov eax, cr4
    or eax, CR4_PAE_BIT
    mov cr4, eax

    ; Enable Long Mode (LME) and No-Execute (NXE) in IA32_EFER MSR (0xC0000080)
    mov ecx, EFER_MSR
    rdmsr
    or eax, (EFER_LONG_MODE_BIT | EFER_NXE_BIT)
    wrmsr

    ; Enable Paging in CR0 to transition into 64-bit Long Mode
    mov eax, cr0
    or eax, CR0_PAGING_BIT
    mov cr0, eax

    ; Load 64-bit GDT descriptor
    lgdt [AP_TRAMPOLINE_BASE + (ap_gdt64_desc - ap_trampoline_start)]

    ; Far jump into 64-bit Long Mode code segment
    jmp dword GDT_CODE64_SEL:(AP_TRAMPOLINE_BASE + (ap_lm64_entry - ap_trampoline_start))

[bits 64]
ap_lm64_entry:
    mov ax, GDT_DATA64_SEL
    mov ds, ax
    mov es, ax
    mov ss, ax
    xor ax, ax
    mov fs, ax
    mov gs, ax

    ; Load assigned kernel stack pointer for this AP core
    mov rsp, [abs AP_TRAMPOLINE_BASE + (ap_stack_val - ap_trampoline_start)]

    ; RDI = core_id argument for System V AMD64 ABI: ap_main(core_id: usize)
    mov rdi, [abs AP_TRAMPOLINE_BASE + (ap_core_id - ap_trampoline_start)]

    ; Transfer control to Rust AP initialization entrypoint
    mov rax, [abs AP_TRAMPOLINE_BASE + (ap_entry_val - ap_trampoline_start)]
    call rax

.ap_halt:
    cli
    hlt
    jmp .ap_halt

; Trampoline Parameter Block
align 8
ap_cr3_val:     dq 0
ap_stack_val:   dq 0
ap_entry_val:   dq 0
ap_core_id:     dq 0
ap_status_flag: dq 0

; Temporary 32-bit GDT
align 16
ap_gdt_start:
    dq 0x0000000000000000
    dq 0x00CF9A000000FFFF
    dq 0x00CF92000000FFFF
ap_gdt_end:

ap_gdt_desc:
    dw ap_gdt_end - ap_gdt_start - 1
    dd AP_TRAMPOLINE_BASE + (ap_gdt_start - ap_trampoline_start)

; 64-bit GDT
align 16
ap_gdt64_start:
    dq 0x0000000000000000
    dq 0x00AF9A000000FFFF
    dq 0x00CF92000000FFFF
ap_gdt64_end:

ap_gdt64_desc:
    dw ap_gdt64_end - ap_gdt64_start - 1
    dd AP_TRAMPOLINE_BASE + (ap_gdt64_start - ap_trampoline_start)

align 16
ap_trampoline_end:
