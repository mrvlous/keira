; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Operating System Kernel
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; 32-Bit Protected Mode Bootstrap Entry Point
;
; This is the initial entry point executed after a Multiboot2-compliant bootloader
; loads the kernel binary into memory. Upon entry, the CPU is operating in 32-bit
; protected mode with the following hardware register contract:
;   - EAX: Multiboot2 magic number verification token (0x36D76289)
;   - EBX: Physical memory address of the Multiboot2 information structure
;   - Paging: Disabled
;   - Interrupts: Disabled
;   - A20 Line: Enabled

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

%ifdef TARGET_ARCH_X86

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

%else

extern setup_page_tables
extern gdt_descriptor
extern _start64

_start:
    ; Step 1: Initialize temporary bootstrap stack pointer
    mov esp, stack_top

    ; Step 2: Preserve Multiboot2 information pointer (EBX) on stack
    push ebx

    ; Step 3: Validate Multiboot2 magic signature in EAX
    cmp eax, MULTIBOOT2_BOOTLOADER
    jne .halt_no_multiboot

    ; Step 4: Construct page tables for 2 MiB identity mapping
    call setup_page_tables

    ; Step 5: Load address of PML4 root table into CR3 register
    extern pml4_table
    mov eax, pml4_table
    mov cr3, eax

    ; Step 6: Enable Physical Address Extension (PAE) in CR4 register
    mov eax, cr4
    or  eax, CR4_PAE_BIT
    mov cr4, eax

    ; Step 7: Enable Long Mode and No-Execute (NXE) in Extended Feature Enable Register (IA32_EFER MSR)
    mov ecx, EFER_MSR
    rdmsr
    or  eax, (EFER_LONG_MODE_BIT | EFER_NXE_BIT)
    wrmsr

    ; Step 8: Enable paging in CR0 register to activate 64-bit long mode
    mov eax, cr0
    or  eax, CR0_PAGING_BIT
    mov cr0, eax

    ; Step 9: Load 64-bit Global Descriptor Table (GDT)
    lgdt [gdt_descriptor]

    ; Step 10: Perform far jump to transfer execution into 64-bit code segment
    jmp GDT_CODE64_SEL:_start64

.halt_no_multiboot:
    ; Display error character 'M' on top-left of VGA text frame buffer (Red background)
    mov dword [VGA_BUFFER_ADDR], 0x4F4D
    jmp .halt

.halt:
    cli
    hlt
    jmp .halt

%endif
