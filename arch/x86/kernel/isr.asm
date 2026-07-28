; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Operating System Kernel
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Interrupt Service Routines (ISRs) and CPU Exception Stubs
;
; Assembly trampolines that preserve register states across hardware interrupts
; and architectural CPU exceptions, invoke high-level C/Rust handlers, and perform
; interrupt returns via `iretq`.

section .text
bits 64

; Save 15 general-purpose registers (120 bytes frame payload)
%macro pushaq 0
    push rax
    push rcx
    push rdx
    push rbx
    push rbp
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
%endmacro

; Restore 15 general-purpose registers
%macro popaq 0
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rbp
    pop rbx
    pop rdx
    pop rcx
    pop rax
%endmacro

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
    push 0
    push %1
    jmp exception_common
%endmacro

%macro exception_err 1
global exception%1
exception%1:
    push %1
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
    pushaq
    mov rdi, rsp
    call exception_dispatcher
    popaq
    add rsp, 16
    iretq

; ISR 32: Programmable Interval Timer (IRQ 0). Preemptive context switch entry.
isr32:
    pushaq
    call pit_handler
    mov rdi, rsp
    call schedule_tick
    mov rsp, rax
    popaq
    iretq

; ISR 33: PS/2 Keyboard (IRQ 1)
isr33:
    pushaq
    call keyboard_handler
    popaq
    iretq

; ISR 44: PS/2 Mouse (IRQ 12)
isr44:
    pushaq
    call mouse_handler
    popaq
    iretq
    