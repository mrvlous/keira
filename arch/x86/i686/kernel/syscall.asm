; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Low-Level System Call Trampolines and Privilege Transitions (i686 32-Bit)
;
; Architectural Role:
;   This assembly module implements hardware privilege ring transitions
;   (Ring 0 Kernel Mode to Ring 3 User Mode) and the legacy
;   software interrupt vector 128 (int 0x80) system call dispatcher for 32-bit x86.
;
; Privilege Level Transition (Ring 0 to Ring 3):
;   The x86 architecture does not permit direct jumps from Ring 0 to Ring 3.
;   Transition is achieved by constructing a synthetic hardware interrupt return
;   frame on the stack and executing iretd:
;     [esp + 16] = User SS   (Selector 0x20 | RPL 3 = 0x23)
;     [esp + 12] = User ESP  (Top of userland stack)
;     [esp + 8]  = EFLAGS    (Bit 9 IF set = 0x202, enabling interrupts in user mode)
;     [esp + 4]  = User CS   (Selector 0x18 | RPL 3 = 0x1B)
;     [esp + 0]  = User EIP  (Entry point address of the user ELF binary)
;
; System Call ABI (Vector 0x80):
;   EAX: System Call Number (vector index).
;   EBX: Argument 1
;   ECX: Argument 2
;   EDX: Argument 3
;   ESI: Argument 4
;   EDI: Argument 5
;   EBP: Argument 6
;   Return Value: EAX (standard POSIX errno / result value).

global jump_to_user
global abort_user_mode
global main_kernel_stack

extern syscall_dispatcher

section .data
align 4
main_kernel_stack: dd 0

section .text
bits 32

; jump_to_user: Lower execution privilege level from Ring 0 to Ring 3
; Calling Convention:
;   cdecl: extern "C" fn jump_to_user(entry_point: u64, user_stack: u64) -> !;
; Stack Layout at Entry:
;   [esp + 0]  = Return address (to kernel caller)
;   [esp + 4]  = entry_point (low 32-bit of u64)
;   [esp + 8]  = entry_point (high 32-bit of u64, unused on 32-bit)
;   [esp + 12] = user_stack  (low 32-bit of u64)
;   [esp + 16] = user_stack  (high 32-bit of u64, unused on 32-bit)
jump_to_user:
    ; Preserve kernel callee-saved registers for recovery via abort_user_mode
    push ebp
    push ebx
    push esi
    push edi

    ; Save current kernel stack pointer to allow graceful return on exit
    mov [main_kernel_stack], esp
    cli

    ; Read entry_point and user_stack parameters (adjusted for 4 pushed registers)
    mov edx, [esp + 20]         ; entry_point (EIP)
    mov ecx, [esp + 28]         ; user_stack  (ESP)

    ; Load segment registers with User Data Segment selector (0x20 | 3 = 0x23)
    mov ax, 0x23
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; Construct Synthetic iretd Frame
    push dword 0x23             ; User SS (0x20 | RPL 3)
    push ecx                    ; User ESP
    push dword 0x202            ; EFLAGS: Bit 1 (reserved) | Bit 9 (IF enabled)
    push dword 0x1B             ; User CS (0x18 | RPL 3)
    push edx                    ; User EIP

    ; Scrub general-purpose registers to eliminate kernel data leakage into Ring 3
    xor eax, eax
    xor ebx, ebx
    xor ecx, ecx
    xor edx, edx
    xor esi, esi
    xor edi, edi
    xor ebp, ebp

    ; Atomic transition to Ring 3 Userland
    iretd

; isr128: Software Interrupt System Call Entry (int 0x80)
; Invoked when userland issues int 0x80.
; Serializes userland arguments, reloads kernel segments, and dispatches to Rust.
global isr128
isr128:
    ; Preserve userland caller arguments and context
    push ebp
    push edi
    push esi
    push edx
    push ecx
    push ebx
    push eax                    ; Preserve syscall number across segment reload

    ; Switch to Kernel Data Segment (0x10) to access kernel memory safely
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    pop eax                     ; Restore syscall number

    ; Marshal Arguments for Rust syscall_dispatcher
    ; Rust signature:
    ;   syscall_dispatcher(n: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, a6: u64)
    ; Push 64-bit zero-extended values (high dword = 0, low dword = reg)
    push dword 0
    push ebp                    ; a6 = EBP
    push dword 0
    push edi                    ; a5 = EDI
    push dword 0
    push esi                    ; a4 = ESI
    push dword 0
    push edx                    ; a3 = EDX
    push dword 0
    push ecx                    ; a2 = ECX
    push dword 0
    push ebx                    ; a1 = EBX
    push dword 0
    push eax                    ; n  = EAX

    call syscall_dispatcher     ; Result returned in EAX
    add esp, 56                 ; Clean 7 * 8 = 56 bytes off stack

    ; Check if process called sys_exit (signaled by magic sentinel 0xDEADBEEF)
    cmp eax, 0xDEADBEEF
    je .exit_user_mode

    ; Reload User Data Segment (0x23)
    mov dx, 0x23
    mov ds, dx
    mov es, dx
    mov fs, dx
    mov gs, dx

    ; Restore userland registers (EAX contains syscall return value)
    pop ebx
    pop ecx
    pop edx
    pop esi
    pop edi
    pop ebp
    iretd

; abort_user_mode: Terminate userland execution and return to kernel caller
.exit_user_mode:
abort_user_mode:
    ; Restore original kernel stack captured during jump_to_user
    mov esp, [main_kernel_stack]

    ; Restore kernel data and stack segments
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax

    ; Restore preserved kernel callee-saved registers
    pop edi
    pop esi
    pop ebx
    pop ebp
    ret
