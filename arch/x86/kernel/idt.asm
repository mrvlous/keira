; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Operating System Kernel
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Interrupt Descriptor Table (IDT) Register Loader

%ifdef TARGET_ARCH_X86

section .text
bits 32

global idt_load
idt_load:
    mov eax, [esp + 4]
    lidt [eax]
    ret

%else

section .text
bits 64

global idt_load
idt_load:
    lidt [rdi]
    ret

%endif
