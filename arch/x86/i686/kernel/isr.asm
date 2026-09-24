; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Interrupt Service Routines (ISRs) and Exception Dispatchers (i686 32-Bit)
;
; Architectural Role:
;   This assembly module implements hardware interrupt stubs, CPU fault/trap
;   exception handlers, and preemptive context switching entry points for
;   32-bit x86 architecture.
;
; Interrupt Frame Layout (Hardware pushed upon interrupt):
;   [esp + 16] = SS     (pushed only if privilege transition occurred)
;   [esp + 12] = ESP    (pushed only if privilege transition occurred)
;   [esp + 8]  = EFLAGS
;   [esp + 4]  = CS
;   [esp + 0]  = EIP
;
; Unified Exception Frame Layout (after stub pushes):
;   [esp + 44] = EFLAGS
;   [esp + 40] = CS
;   [esp + 36] = EIP
;   [esp + 32] = Error Code (pushed by CPU or dummy 0)
;   [esp + 28] = Exception / Interrupt Vector Number
;   [esp + 0..27] = EDI, ESI, EBP, ESP, EBX, EDX, ECX, EAX (via pushad)

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

; Exception Generator Macros
; exception_no_err:
;   Used for exceptions where CPU does NOT automatically push an error code.
;   Pushes a dummy error code (0) to maintain uniform stack frame alignment.
%macro exception_no_err 1
global exception%1
exception%1:
    push dword 0                ; Push dummy error code
    push dword %1               ; Push interrupt vector number
    jmp exception_common
%endmacro

; exception_err:
;   Used for exceptions where CPU pushes a hardware error code onto the stack
;   (#DF, #TS, #NP, #SS, #GP, #PF, #AC, #CP).
%macro exception_err 1
global exception%1
exception%1:
    push dword %1               ; Push interrupt vector number (error code already on stack)
    jmp exception_common
%endmacro

; Standard CPU Exception Vectors (0 - 31)
exception_no_err 0              ; #DE: Divide-by-Zero Error
exception_no_err 1              ; #DB: Debug Exception
exception_no_err 2              ; NMI: Non-Maskable Interrupt
exception_no_err 3              ; #BP: Breakpoint (INT3)
exception_no_err 4              ; #OF: Overflow (INTO)
exception_no_err 5              ; #BR: BOUND Range Exceeded
exception_no_err 6              ; #UD: Invalid Opcode (Undefined Opcode)
exception_no_err 7              ; #NM: Device Not Available (No Math Coprocessor)
exception_err    8              ; #DF: Double Fault (Error Code = 0)
exception_no_err 9              ; Coprocessor Segment Overrun (Reserved)
exception_err    10             ; #TS: Invalid TSS
exception_err    11             ; #NP: Segment Not Present
exception_err    12             ; #SS: Stack-Segment Fault
exception_err    13             ; #GP: General Protection Fault
exception_err    14             ; #PF: Page Fault (CR2 contains faulting linear address)
exception_no_err 15             ; Reserved / Spurious Interrupt
exception_no_err 16             ; #MF: x87 FPU Floating-Point Error (Math Fault)
exception_err    17             ; #AC: Alignment Check
exception_no_err 18             ; #MC: Machine Check
exception_no_err 19             ; #XM: SIMD Floating-Point Exception
exception_no_err 20             ; #VE: Virtualization Exception
exception_err    21             ; #CP: Control Protection Exception
exception_no_err 22             ; Reserved (22-31)
exception_no_err 23
exception_no_err 24
exception_no_err 25
exception_no_err 26
exception_no_err 27
exception_no_err 28
exception_no_err 29
exception_no_err 30
exception_no_err 31

; Common CPU Exception Handler
; Serializes full register state, invokes Rust dispatcher, and performs atomic
; return via iretd.
global exception_common
exception_common:
    pushad                      ; Save EAX, ECX, EDX, EBX, ESP, EBP, ESI, EDI (32 bytes)
    push esp                    ; Pass pointer to full ExceptionFrame as first cdecl argument
    call exception_dispatcher   ; Invoke Rust panic/fault dispatcher
    add esp, 4                  ; Clean argument pointer off stack
    popad                       ; Restore general-purpose registers
    add esp, 8                  ; Pop vector number and error code
    iretd                       ; Atomically restore EIP, CS, EFLAGS (and ESP, SS if Ring 3)

; ISR 32: Programmable Interval Timer (IRQ 0) and Preemptive Task Switcher
; Handles timer interrupt tick and performs preemptive round-robin scheduling.
; If schedule_tick selects a new thread, it returns the target thread's ESP.
isr32:
    pushad                      ; Preserve current task general-purpose registers
    call pit_handler            ; Send EOI to PIC/APIC and increment uptime tick

    mov eax, esp                ; Current task stack pointer
    push dword 0                ; Argument 2: core_id = 0
    push eax                    ; Argument 1: current ESP
    call schedule_tick          ; schedule_tick(current_esp: *mut u8, core_id: usize) -> *mut u8
    add esp, 8                  ; Clean arguments
    mov esp, eax                ; Switch execution stack to target task (context switch)

    popad                       ; Restore general-purpose registers of newly scheduled task
    iretd                       ; Return to execution in chosen task

; ISR 33: PS/2 Keyboard Controller (IRQ 1)
; Reads scancodes from port 0x60 and forwards to Rust keyboard buffer.
isr33:
    pushad
    call keyboard_handler
    popad
    iretd

; ISR 44: PS/2 Mouse Controller (IRQ 12)
; Reads auxiliary device mouse packets from port 0x60.
isr44:
    pushad
    call mouse_handler
    popad
    iretd
