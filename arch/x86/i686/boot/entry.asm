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
; Architectural Role:
;   This assembly routine is the primary entry point executed when a Multiboot2-
;   compliant bootloader transfers control to Keira on 32-bit x86 (i686) hardware.
;
; Bootloader Handover State (Guaranteed by Multiboot2 Specification):
;   EAX: Must contain the magic signature 0x36D76289 (MULTIBOOT2_BOOTLOADER).
;   EBX: 32-bit physical address pointing to the Multiboot2 Information Structure.
;   CS:  32-bit read/execute code segment with offset 0 and limit 0xFFFFFFFF.
;   DS, ES, FS, GS, SS: 32-bit read/write data segments with offset 0 and limit 0xFFFFFFFF.
;   A20 Gate: Fully enabled.
;   CR0: Protection Enable bit set (PE = 1), Paging disabled (PG = 0).
;   Interrupts: Disabled (EFLAGS.IF = 0).
;   ESP: Undefined (the kernel MUST establish its own stack immediately).
;
; Execution Pipeline:
;   1. Initialize temporary kernel bootstrap stack.
;   2. Validate Multiboot2 handshake signature in EAX.
;   3. Install Keira canonical 32-bit Global Descriptor Table (GDT).
;   4. Reload segment registers and perform far jump to flush CPU pipeline.
;   5. Push Multiboot2 info pointer and invoke Rust entrypoint (kernel_main).
;   6. Graceful halt loop if kernel_main ever returns.

%include "constants.inc"

; Bootstrap Stack Allocation (.bss section)
; Allocates 32 KiB of uninitialized memory aligned to a 16-byte boundary
; to comply with standard 32-bit x86 ABI alignment requirements.
section .bss
align 16

stack_bottom:
    resb KERNEL_STACK_SIZE
global stack_top
stack_top:

; Kernel Bootstrap Code Section (.text)
section .text
bits 32
global _start

extern kernel_main
extern gdt_descriptor

_start:
    ; Step 1: Initialize Temporary Bootstrap Stack
    ; Stack grows downward on x86; point ESP to highest address of the buffer.
    mov esp, stack_top

    ; Step 2: Validate Multiboot2 Bootloader Handshake
    ; Confirm that the kernel was booted by a certified Multiboot2 loader.
    ; If EAX != 0x36D76289, abort immediately to error display handler.
    cmp eax, MULTIBOOT2_BOOTLOADER
    jne .halt_no_multiboot

    ; Step 3: Install Kernel Global Descriptor Table (GDT)
    ; Load Keira internal 32-bit GDT to replace bootloader-provided segments.
    lgdt [gdt_descriptor]

    ; Far jump to reload Code Segment (CS) with selector 0x08 (Kernel Code)
    ; and serialize CPU instruction prefetch pipeline.
    jmp 0x08:.reload_cs

.reload_cs:
    ; Step 4: Reload Data Segment Selectors
    ; Selector 0x10 points to Kernel Data Segment (Base 0, Limit 4GB, Writable, DPL 0).
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    ; Step 5: Transfer Control to Rust Kernel (kernel_main)
    ; Multiboot2 information structure pointer in EBX is passed as the first
    ; argument via standard cdecl calling convention (pushed onto stack).
    push ebx
    call kernel_main

    ; Step 6: Infinite Halt Loop (CPU Idle Fallback)
    ; In the event that kernel_main exits, ensure CPU enters a low-power
    ; halted state with interrupts permanently disabled.
.halt:
    cli
    hlt
    jmp .halt

    ; Error Handler: Invalid Multiboot2 Signature
    ; Emits red "M!" glyphs directly into the top-left VGA text buffer to provide
    ; immediate visual diagnostics without relying on any kernel subsystems.
.halt_no_multiboot:
    mov dword [VGA_BUFFER_ADDR], 0x4F4D  ; 'M' in white on red attribute
    jmp .halt
