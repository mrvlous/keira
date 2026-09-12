; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Interrupt Service Routines (ISRs) and CPU Exception Handlers (i686 32-Bit)

section .text
bits 32

global isr32
global isr33
global isr44

extern isr_handler
extern keyboard_handler
extern mouse_handler
extern pit_handler
extern schedule_tick
extern exception_dispatcher

%macro exception_no_err 1
global exception%1
exception%1:
    push dword 0
    push dword %1
    jmp exception_common
%endmacro

%macro exception_err 1
global exception%1
exception%1:
    push dword %1
    jmp exception_common
%endmacro

exception_no_err 0
exception_no_err 1
exception_no_err 2
exception_no_err 3
exception_no_err 4
exception_no_err 5
exception_no_err 6
exception_no_err 7
exception_err    8
exception_no_err 9
exception_err    10
exception_err    11
exception_err    12
exception_err    13
exception_err    14
exception_no_err 15
exception_no_err 16
exception_err    17
exception_no_err 18
exception_no_err 19
exception_no_err 20
exception_err    21
exception_no_err 22
exception_no_err 23
exception_no_err 24
exception_no_err 25
exception_no_err 26
exception_no_err 27
exception_no_err 28
exception_no_err 29
exception_no_err 30
exception_no_err 31

global exception_common
exception_common:
    pushad
    push esp
    call exception_dispatcher
    add esp, 4
    popad
    add esp, 8
    iretd

; ISR 32: PIT Timer (IRQ 0)
isr32:
    pushad
    call pit_handler
    mov eax, esp
    push dword 0
    push eax
    call schedule_tick
    add esp, 8
    mov esp, eax
    popad
    iretd

; ISR 33: PS/2 Keyboard (IRQ 1)
isr33:
    pushad
    call keyboard_handler
    popad
    iretd

; ISR 44: PS/2 Mouse (IRQ 12)
isr44:
    pushad
    call mouse_handler
    popad
    iretd
