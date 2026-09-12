; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Interrupt Descriptor Table (IDT) Register Loader (i686 32-Bit)

section .text
bits 32

global idt_load
idt_load:
    mov eax, [esp + 4]
    lidt [eax]
    ret
