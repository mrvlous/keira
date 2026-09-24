; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Interrupt Descriptor Table (IDT) Register Loader (i686 32-Bit)
;
; Architectural Role:
;   This assembly function invokes the privileged lidt instruction to load
;   the CPU Interrupt Descriptor Table Register (IDTR) on 32-bit x86 systems.
;
; IDTR Register Structure (6 bytes total):
;   Bytes 0-1: IDT Limit (size of IDT in bytes minus 1, e.g., 256 * 8 - 1 = 2047).
;   Bytes 2-5: 32-bit linear base address of the IDT gate array.
;
; Preconditions:
;   Caller must pass the linear address of a valid 6-byte IDTR descriptor
;   pointer as the first argument on the stack.
;   All referenced Interrupt and Trap Gate descriptors must be initialized
;   in memory before this instruction executes.

section .text
bits 32

; idt_load: Load IDTR from passed pointer
; Calling Convention:
;   cdecl: extern "C" fn idt_load(ptr: *const IdtPointer);
; Stack Frame:
;   [esp + 4] = 32-bit address of the 6-byte IDTR descriptor.
global idt_load
idt_load:
    mov eax, [esp + 4]          ; Retrieve IdtPointer memory address from stack
    lidt [eax]                  ; Load IDTR with table limit and linear base
    ret
