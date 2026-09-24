; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; 32-Bit to 64-Bit Long Mode Trampoline Entry Point (x86_64)
;
; Architectural Role:
;   This assembly module executes the critical processor mode transition from
;   legacy 32-bit Protected Mode (handed over by GRUB2 / Multiboot2) to 64-bit
;   IA-32e Long Mode for the Bootstrap Processor (BSP).
;
; Hardware Transition Pipeline:
;   1. Stack Setup: Initialize temporary bootstrap stack for 32-bit execution.
;   2. Multiboot2 Handshake: Verify bootloader signature in EAX; preserve EBX.
;   3. Paging Construction: Invoke setup_page_tables to build identity mapping.
;   4. CR3 Loading: Load linear base of the PML4 table into Control Register 3.
;   5. PAE Activation: Set CR4.PAE (Physical Address Extension bit 5).
;   6. Long Mode Enable: Set IA32_EFER.LME (bit 8) and NXE (bit 11) via MSR write.
;   7. Paging Activation: Set CR0.PG (bit 31). This activates IA-32e Compatibility Mode.
;   8. 64-Bit GDT Installation: Load 64-bit GDT with code/data segment descriptors.
;   9. Long Mode Far Jump: Execute 64-bit far jump (jmp 0x08:_start64) to enter
;      pure 64-bit submode and serialize the CPU prefetch queue.

%include "constants.inc"

; Bootstrap Stack Allocation (.bss section)
section .bss
align 16

stack_bottom:
    resb KERNEL_STACK_SIZE
global stack_top
stack_top:

; 32-Bit Bootstrap Code Section (.text)
section .text
bits 32
global _start

extern setup_page_tables
extern gdt_descriptor
extern _start64
extern pml4_table

_start:
    ; Step 1: Establish Temporary Bootstrap Execution Stack
    mov esp, stack_top

    ; Step 2: Preserve Multiboot2 Pointer and Validate Handshake Signature
    ; Preserve Multiboot2 Information Structure pointer across early setup
    push ebx

    ; Compare EAX against expected Multiboot2 bootloader signature (0x36D76289)
    cmp eax, MULTIBOOT2_BOOTLOADER
    jne .halt_no_multiboot

    ; Step 3: Populate 4-Level Early Page Tables
    ; Constructs 1 GiB identity-mapped page hierarchy using 2 MiB huge pages
    call setup_page_tables

    ; Step 4: Load Page Map Level 4 (PML4) Base Address into CR3
    mov eax, pml4_table
    mov cr3, eax

    ; Step 5: Enable Physical Address Extension (PAE) in CR4
    ; PAE (CR4 bit 5) is mandatory before Long Mode can be engaged.
    mov eax, cr4
    or  eax, CR4_PAE_BIT
    mov cr4, eax

    ; Step 6: Enable Long Mode (LME) and No-Execute (NXE) in IA32_EFER MSR
    ; MSR 0xC0000080 (IA32_EFER):
    ;   Bit 8  (LME): Long Mode Enable
    ;   Bit 11 (NXE): No-Execute Enable (hardware execute disable)
    mov ecx, EFER_MSR
    rdmsr
    or  eax, (EFER_LONG_MODE_BIT | EFER_NXE_BIT)
    wrmsr

    ; Step 7: Enable Hardware Paging (CR0.PG) to Engage Long Mode
    ; Once CR0.PG is set while EFER.LME is active, CPU enters IA-32e mode.
    mov eax, cr0
    or  eax, CR0_PAGING_BIT
    mov cr0, eax

    ; Step 8: Load 64-Bit Global Descriptor Table (GDTR)
    lgdt [gdt_descriptor]

    ; Step 9: Long Mode Far Jump into Pure 64-Bit Mode
    ; Segment selector 0x08 points to 64-bit Kernel Code Descriptor (CS.L = 1).
    jmp GDT_CODE64_SEL:_start64

; Error Handler: Invalid Multiboot2 Signature
.halt_no_multiboot:
    mov dword [VGA_BUFFER_ADDR], 0x4F4D  ; 'M' in white-on-red text mode
    jmp .halt

; Terminal Halt Loop
.halt:
    cli
    hlt
    jmp .halt
