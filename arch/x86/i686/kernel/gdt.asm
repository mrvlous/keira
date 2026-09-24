; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Global Descriptor Table (GDT) Configuration and Management (i686 32-Bit)
;
; Architectural Role:
;   This assembly module defines the 32-bit flat memory model GDT and segment
;   reloading procedures for i686 architecture in Keira.
;
; Segment Model (Flat Protected Mode):
;   All code and data segments span the complete 4 GiB linear address space
;   (Base = 0x00000000, Limit = 0xFFFFFFFF, Granularity = 4 KiB).
;   Memory protection is delegated entirely to the hardware Paging unit.
;
; Segment Selector Index Layout:
;   Selector 0x00: Mandatory CPU Null Descriptor
;   Selector 0x08: Kernel Code Segment (Ring 0, Execute/Read)
;   Selector 0x10: Kernel Data Segment (Ring 0, Read/Write)
;   Selector 0x18: User Code Segment   (Ring 3, Execute/Read)
;   Selector 0x20: User Data Segment   (Ring 3, Read/Write)
;   Selector 0x28: Task State Segment (TSS, 32-bit Available)
;
; Descriptor Bit Field Layout (8 bytes):
;   [Limit 15:0] [Base 15:0] [Base 23:16] [Access Byte] [Flags & Limit 19:16] [Base 31:24]
;
; Access Byte Encoding:
;   Bit 7 (P):   Present (1 = in memory)
;   Bit 6-5(DPL):Descriptor Privilege Level (00b = Kernel, 11b = Userland)
;   Bit 4 (S):   Descriptor Type (1 = Code/Data, 0 = System/TSS)
;   Bit 3 (E):   Executable (1 = Code, 0 = Data)
;   Bit 2 (DC):  Direction / Conforming bit
;   Bit 1 (RW):  Readable (for code) / Writable (for data)
;   Bit 0 (A):   Accessed bit (set by CPU on memory read/write)

section .data
align 16

global gdt_start
gdt_start:
    ; Selector 0x00: Null Descriptor
    ; Required by CPU architecture. Accessing through selector 0x00 causes #GP.
    dq 0x0000000000000000

    ; Selector 0x08: Kernel Code Segment (Ring 0)
    ; Base: 0x0, Limit: 0xFFFFF (4 GiB with Granularity = 1)
    ; Access: 10011010b -> Present, DPL 0, Code/Data, Executable, Readable
    ; Flags:  11001111b -> Granularity 4 KiB, 32-bit Protected Mode operation
    dw 0xFFFF                   ; Limit (bits 0-15)
    dw 0x0000                   ; Base  (bits 0-15)
    db 0x00                     ; Base  (bits 16-23)
    db 10011010b                ; Access byte (P=1, DPL=00, S=1, E=1, DC=0, RW=1, A=0)
    db 11001111b                ; Flags (G=1, D=1, L=0, AVL=0) | Limit (bits 16-19)
    db 0x00                     ; Base  (bits 24-31)

    ; Selector 0x10: Kernel Data Segment (Ring 0)
    ; Base: 0x0, Limit: 0xFFFFF (4 GiB with Granularity = 1)
    ; Access: 10010010b -> Present, DPL 0, Code/Data, Data Writable
    dw 0xFFFF                   ; Limit (bits 0-15)
    dw 0x0000                   ; Base  (bits 0-15)
    db 0x00                     ; Base  (bits 16-23)
    db 10010010b                ; Access byte (P=1, DPL=00, S=1, E=0, DC=0, RW=1, A=0)
    db 11001111b                ; Flags (G=1, D=1, L=0, AVL=0) | Limit (bits 16-19)
    db 0x00                     ; Base  (bits 24-31)

    ; Selector 0x18: User Code Segment (Ring 3)
    ; Base: 0x0, Limit: 0xFFFFF (4 GiB with Granularity = 1)
    ; Access: 11111010b -> Present, DPL 3 (Userland), Code/Data, Executable, Readable
    dw 0xFFFF                   ; Limit (bits 0-15)
    dw 0x0000                   ; Base  (bits 0-15)
    db 0x00                     ; Base  (bits 16-23)
    db 11111010b                ; Access byte (P=1, DPL=11, S=1, E=1, DC=0, RW=1, A=0)
    db 11001111b                ; Flags (G=1, D=1, L=0, AVL=0) | Limit (bits 16-19)
    db 0x00                     ; Base  (bits 24-31)

    ; Selector 0x20: User Data Segment (Ring 3)
    ; Base: 0x0, Limit: 0xFFFFF (4 GiB with Granularity = 1)
    ; Access: 11110010b -> Present, DPL 3 (Userland), Code/Data, Data Writable
    dw 0xFFFF                   ; Limit (bits 0-15)
    dw 0x0000                   ; Base  (bits 0-15)
    db 0x00                     ; Base  (bits 16-23)
    db 11110010b                ; Access byte (P=1, DPL=11, S=1, E=0, DC=0, RW=1, A=0)
    db 11001111b                ; Flags (G=1, D=1, L=0, AVL=0) | Limit (bits 16-19)
    db 0x00                     ; Base  (bits 24-31)

    ; Selector 0x28: Task State Segment (TSS) Descriptor (32-Bit)
    ; Type 10001001b -> Present, DPL 0, System, 32-bit TSS (Available).
    ; Populated dynamically at runtime with linear base address of the TSS.
global tss_descriptor
tss_descriptor:
    dw 0x0000                   ; TSS Limit (bits 0-15) - configured at runtime
    dw 0x0000                   ; TSS Base  (bits 0-15)
    db 0x00                     ; TSS Base  (bits 16-23)
    db 10001001b                ; Access (P=1, DPL=00, S=0, Type=1001b: 32-bit TSS Available)
    db 0x00                     ; Flags & Limit (bits 16-19)
    db 0x00                     ; TSS Base  (bits 24-31)

gdt_end:

; GDT Pseudo-Descriptor (Structure passed to lgdt instruction)
global gdt_descriptor
gdt_descriptor:
    dw gdt_end - gdt_start - 1  ; Table limit (size in bytes minus 1)
    dd gdt_start                ; 32-bit linear base address of GDT

section .text
bits 32

; reload_gdt: Reload CPU Global Descriptor Table Register (GDTR)
; Purpose:
;   Installs Keira GDT and flushes segment registers across the CPU pipeline.
; Calling Convention:
;   cdecl (no arguments, preserves EBP/EBX/ESI/EDI).
global reload_gdt
reload_gdt:
    lgdt [gdt_descriptor]       ; Load GDTR with table base and limit

    ; Far jump to reload Code Segment (CS) with selector 0x08
    jmp 0x08:.reload_cs

.reload_cs:
    ; Reload all data and stack segment registers with Kernel Data Selector 0x10
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    ret

; load_tss: Load Task Register (TR)
; Purpose:
;   Loads selector 0x28 into the CPU Task Register via the ltr instruction.
;   Ensures processor can look up ESP0 during Ring 3 to Ring 0 privilege
;   transitions (interrupts and system calls).
global load_tss
load_tss:
    mov ax, 0x28                ; TSS Segment Selector (Index 5, RPL 0)
    ltr ax                      ; Load Task Register
    ret
