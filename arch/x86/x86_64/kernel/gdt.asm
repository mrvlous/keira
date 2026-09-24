; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Global Descriptor Table (GDT) Configuration and Management (x86_64)
;
; Architectural Role:
;   This assembly module defines the 64-bit GDT table, segment descriptor layout,
;   and privileged loading routines (lgdt, ltr) for x86_64 architecture in Keira.
;
; 64-Bit Segmentation Mechanics:
;   In IA-32e Long Mode, traditional segmentation is largely disabled:
;   Code and data segment bases are treated as 0x0000000000000000 regardless
;   of the descriptor base field (except FS/GS base via MSRs).
;   Segment limits are ignored (the entire 64-bit canonical address space is accessible).
;   Descriptors still enforce privilege rings (DPL 0 vs DPL 3), code execution
;   permissions, and long mode submode status (CS.L = 1 for 64-bit mode).
;
; Selector Layout (sysret and syscall Compatibility):
;   The x86 sysret instruction assumes a rigid descriptor ordering relative
;   to the base selector defined in the IA32_STAR MSR (bits 63:48):
;     Selector 0x00: Null Descriptor
;     Selector 0x08: 64-bit Kernel Code (DPL 0)
;     Selector 0x10: 64-bit Kernel Data (DPL 0)
;     Selector 0x18: 64-bit User Data   (DPL 3)
;     Selector 0x20: 64-bit User Data   (DPL 3, sysret stack target)
;     Selector 0x28: 64-bit User Code   (DPL 3, sysret code target)
;     Selector 0x30: Task State Segment (TSS, 16 bytes: 0x30 and 0x38)
;
; 64-Bit TSS Descriptor Architecture (16 Bytes):
;   Unlike legacy 8-byte descriptors, system descriptors (such as the TSS)
;   are expanded to 16 bytes in Long Mode to hold a full 64-bit linear base address.
;   Bytes 0-7:  Standard descriptor fields (Limit, Base 0..23, Type, Base 24..31).
;   Bytes 8-15: Upper 32 bits of Base address (Base 32..63) and reserved zero bytes.

section .data
align 16

global gdt_start
gdt_start:
    ; Selector 0x00: Mandatory CPU Null Descriptor
    dq 0x0000000000000000

    ; Selector 0x08: 64-Bit Kernel Code Segment (Ring 0)
    ; Access: 10011010b -> Present, DPL 0, Code/Data, Executable, Readable
    ; Flags:  10101111b -> Long Mode (L=1), Default Oper Size (D=0)
    dw 0xFFFF                   ; Segment Limit (0-15)
    dw 0x0000                   ; Base Address  (0-15)
    db 0x00                     ; Base Address  (16-23)
    db 10011010b                ; Access byte (P=1, DPL=00, S=1, E=1, DC=0, RW=1, A=0)
    db 10101111b                ; Flags (G=1, D=0, L=1, AVL=0) | Limit (16-19)
    db 0x00                     ; Base Address  (24-31)

    ; Selector 0x10: 64-Bit Kernel Data Segment (Ring 0)
    ; Access: 10010010b -> Present, DPL 0, Code/Data, Writable
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 10010010b
    db 11001111b                ; Flags (G=1, D=1, L=0, AVL=0) | Limit (16-19)
    db 0x00

    ; Selector 0x18: 64-Bit User Data Segment (Ring 3)
    ; Access: 11110010b -> Present, DPL 3 (Userland), Code/Data, Writable
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 11110010b
    db 11001111b
    db 0x00

    ; Selector 0x20: 64-Bit User Data Segment (sysret stack target)
    ; Matched by sysret when returning from 64-bit system calls to Ring 3.
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 11110010b
    db 11001111b
    db 0x00

    ; Selector 0x28: 64-Bit User Code Segment (sysret code target)
    ; Access: 11111010b -> Present, DPL 3, Code/Data, Executable, Readable
    ; Flags:  10101111b -> Long Mode (L=1), 64-bit operand execution
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 11111010b
    db 10101111b
    db 0x00

    ; Selector 0x30: Task State Segment (TSS) Descriptor (16 Bytes Total)
    ; In x86_64, system descriptors span 16 consecutive bytes (2 GDT slots).
    ; Type 10001001b = Present, DPL 0, System, 64-bit TSS (Available).
    ; Populated dynamically at runtime by Rust GDT setup routine.
global tss_descriptor
tss_descriptor:
    dw 0x0000                   ; TSS Limit (0-15)
    dw 0x0000                   ; TSS Base  (0-15)
    db 0x00                     ; TSS Base  (16-23)
    db 10001001b                ; Access byte (P=1, DPL=00, S=0, Type=1001b)
    db 0x00                     ; Flags & Limit (16-19)
    db 0x00                     ; TSS Base  (24-31)
    dd 0x00000000               ; TSS Base  (32-63, upper 32 bits)
    dd 0x00000000               ; Reserved (Must be 0)

gdt_end:

; GDT Pseudo-Descriptor (Structure loaded into GDTR)
global gdt_descriptor
gdt_descriptor:
    dw gdt_end - gdt_start - 1  ; Limit (Table size in bytes minus 1)
    dq gdt_start                ; 64-bit linear base address of GDT

section .text
bits 64

; reload_gdt: Reload CPU Global Descriptor Table Register (GDTR)
; Uses RIP-relative addressing to load updated GDT descriptor into GDTR.
global reload_gdt
reload_gdt:
    lgdt [rel gdt_descriptor]
    ret

; load_tss: Load Task Register (TR)
; Loads Selector 0x30 into Task Register via ltr.
; Enables hardware privilege level 0 stack switching (TSS.RSP0) upon interrupts.
global load_tss
load_tss:
    mov ax, 0x30                ; TSS Segment Selector (Index 6, RPL 0)
    ltr ax                      ; Load Task Register
    ret
