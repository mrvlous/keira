; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Operating System Kernel
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Global Descriptor Table (GDT) and Segment Layout Configuration
;
; Layout:
;   0x00: Null Descriptor
;   0x08: 64-bit Kernel Code Segment (DPL 0)
;   0x10: 64-bit Kernel Data Segment (DPL 0)
;   0x18: 64-bit User Data Segment (DPL 3)
;   0x20: 64-bit User Data Segment (DPL 3) (Required by sysret: STAR[63:48]+8)
;   0x28: 64-bit User Code Segment (DPL 3) (Required by sysret: STAR[63:48]+16)
;   0x30: Task State Segment (TSS) Descriptor (16-byte System Descriptor)

section .data
align 16

global gdt_start
gdt_start:

; Selector 0x00: Null Descriptor
    dq 0x0000000000000000

; Selector 0x08: 64-bit Kernel Code Segment (DPL 0)
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 10011010b
    db 10101111b
    db 0x00

; Selector 0x10: 64-bit Kernel Data Segment (DPL 0)
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 10010010b
    db 11001111b
    db 0x00

; Selector 0x18: 64-bit User Data Segment (DPL 3)
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 11110010b
    db 11001111b
    db 0x00

; Selector 0x20: 64-bit User Data Segment (DPL 3) - sysret target stack selector
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 11110010b
    db 11001111b
    db 0x00

; Selector 0x28: 64-bit User Code Segment (DPL 3) - sysret target code selector
    dw 0xFFFF
    dw 0x0000
    db 0x00
    db 11111010b
    db 10101111b
    db 0x00

; Selectors 0x30 - 0x38: Dynamic 64-bit TSS Descriptor (16 bytes)
global tss_descriptor
tss_descriptor:
    dw 0x0000
    dw 0x0000
    db 0x00
    db 10001001b
    db 0x00
    db 0x00
    dd 0x00000000
    dd 0x00000000

gdt_end:

global gdt_descriptor
gdt_descriptor:
    dw gdt_end - gdt_start - 1
    dq gdt_start

section .text
bits 64

global reload_gdt
reload_gdt:
    lgdt [rel gdt_descriptor]
    ret

global load_tss
load_tss:
    mov ax, 0x30
    ltr ax
    ret
