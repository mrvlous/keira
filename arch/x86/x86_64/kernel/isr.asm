; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Interrupt Service Routines (ISRs) and Exception Dispatchers (x86_64)
;
; Architectural Role:
;   This assembly module implements the low-level interrupt service routines,
;   CPU fault/trap exception entrypoints, and preemptive context switching stubs
;   for 64-bit x86_64 architecture.
;
; Hardware Interrupt Stack Frame (Pushed unconditionally by CPU upon interrupt):
;   [rsp + 32] = SS      (Userland stack segment selector)
;   [rsp + 24] = RSP     (Userland stack pointer)
;   [rsp + 16] = RFLAGS  (CPU flags register)
;   [rsp + 8]  = CS      (Code segment selector; bits 0-1 RPL denote privilege level)
;   [rsp + 0]  = RIP     (Instruction pointer where interrupt occurred)
;
; SwapGS Protocol and Race-Free Re-entrancy:
;   If interrupt originated in Ring 3 (Userland, where CS & 3 != 0), the kernel
;   MUST execute swapgs to restore the kernel per-CPU GS base pointer.
;   If interrupt originated in Ring 0 (Kernel Mode, where CS & 3 == 0), swapgs
;   MUST NOT be executed to avoid corrupting the active kernel GS context.
;
; Unified Exception Frame Layout (136 bytes total on stack):
;   [rsp + 128] = Hardware Error Code (or dummy 0)
;   [rsp + 120] = Interrupt Vector Number (0 - 255)
;   [rsp + 0..119] = 15 General Purpose Registers (RAX, RCX, RDX, RBX, RBP,
;                    RSI, RDI, R8, R9, R10, R11, R12, R13, R14, R15 via pushaq)

section .text
bits 64

; Register Preservation Macros (pushaq / popaq)
; Saves all 15 general-purpose registers (excluding RSP, which is tracked separately).
; Total stack footprint: 15 * 8 = 120 bytes.
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

; Exception Stub Generator Macros
%macro exception_no_err 1
global exception%1
exception%1:
    push 0                      ; Dummy error code for uniform alignment
    push %1                     ; Vector number
    jmp exception_common
%endmacro

%macro exception_err 1
global exception%1
exception%1:
    push %1                     ; Vector number (hardware error code already pushed)
    jmp exception_common
%endmacro

; CPU Exception Vectors (0 - 31)
exception_no_err 0              ; #DE: Divide-by-Zero
exception_no_err 1              ; #DB: Debug
exception_no_err 2              ; NMI: Non-Maskable Interrupt
exception_no_err 3              ; #BP: Breakpoint (INT3)
exception_no_err 4              ; #OF: Overflow
exception_no_err 5              ; #BR: Bound Range Exceeded
exception_no_err 6              ; #UD: Invalid Opcode
exception_no_err 7              ; #NM: Device Not Available
exception_err    8              ; #DF: Double Fault
exception_no_err 9              ; Coprocessor Segment Overrun
exception_err    10             ; #TS: Invalid TSS
exception_err    11             ; #NP: Segment Not Present
exception_err    12             ; #SS: Stack-Segment Fault
exception_err    13             ; #GP: General Protection Fault
exception_err    14             ; #PF: Page Fault (CR2 contains faulting linear address)
exception_no_err 15             ; Reserved
exception_no_err 16             ; #MF: x87 Floating-Point Exception
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

; Common Exception Dispatcher Routine
global exception_common
exception_common:
    ; Check interrupted CS selector: [rsp + 24] holds CS after vector and error code
    test byte [rsp + 24], 3
    jz .exc_entry_kernel
    swapgs                      ; Interrupted from Ring 3: activate kernel GS base
.exc_entry_kernel:
    pushaq                      ; Push 15 GPRs (120 bytes)

    ; Pass pointer to full ExceptionFrame in RDI per System V AMD64 ABI
    mov rdi, rsp
    call exception_dispatcher   ; Rust exception handling dispatch

    popaq                       ; Restore GPRs
    add rsp, 16                 ; Pop vector number and error code

    ; Check interrupted CS selector before return
    test byte [rsp + 8], 3
    jz .exc_exit_kernel
    swapgs                      ; Returning to Ring 3: restore user GS base
.exc_exit_kernel:
    iretq                       ; Atomically restore RIP, CS, RFLAGS, RSP, SS

; ISR 32: PIT Timer (IRQ 0) - Preemptive Context Switch Entrypoint
isr32:
    test byte [rsp + 8], 3
    jz .pit_entry_kernel
    swapgs
.pit_entry_kernel:
    pushaq
    call pit_handler            ; Increment timer ticks and issue PIC/APIC EOI

    ; Invoke scheduler: schedule_tick(current_rsp: *mut u8) -> *mut u8
    mov rdi, rsp
    call schedule_tick
    mov rsp, rax                ; Context switch: switch stack pointer to next task

    popaq
    test byte [rsp + 8], 3
    jz .pit_exit_kernel
    swapgs
.pit_exit_kernel:
    iretq

; ISR 33: PS/2 Keyboard (IRQ 1)
isr33:
    test byte [rsp + 8], 3
    jz .kbd_entry_kernel
    swapgs
.kbd_entry_kernel:
    pushaq
    call keyboard_handler
    popaq
    test byte [rsp + 8], 3
    jz .kbd_exit_kernel
    swapgs
.kbd_exit_kernel:
    iretq

; ISR 44: PS/2 Mouse (IRQ 12)
isr44:
    test byte [rsp + 8], 3
    jz .mouse_entry_kernel
    swapgs
.mouse_entry_kernel:
    pushaq
    call mouse_handler
    popaq
    test byte [rsp + 8], 3
    jz .mouse_exit_kernel
    swapgs
.mouse_exit_kernel:
    iretq
