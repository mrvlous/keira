; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Operating System Kernel
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Interrupt Descriptor Table (IDT) Register Loader
;
; Invokes the `lidt` instruction to inform the CPU of the IDT location and limit.

section .text
bits 64

global idt_load

; idt_load - Loads the IDT structure into IDTR register
; RDI: Linear physical address of idt_ptr structure
idt_load:
    lidt [rdi]
    ret
