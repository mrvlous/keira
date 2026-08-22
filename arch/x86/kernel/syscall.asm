; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Operating System Kernel
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Low-Level System Call Trampolines and User Mode Privilege Transitions
;
; Initializes system call Model-Specific Registers (IA32_STAR, IA32_LSTAR, IA32_SFMASK),
; provides high-speed `syscall` entry stubs, and manages Ring 0 to Ring 3 privilege switches.

global init_syscall_msrs
global syscall_handler_asm
global jump_to_user

global user_rsp_temp
global kernel_stack_temp

extern syscall_dispatcher

section .data
align 8
user_rsp_temp:     dq 0
kernel_stack_temp: dq 0

section .text
bits 64

; init_syscall_msrs - Configure Model-Specific Registers for fast system calls
init_syscall_msrs:
    ; Enable System Call Extensions (SCE) and No-Execute (NXE) in EFER MSR (0xC0000080)
    mov ecx, 0xC0000080
    rdmsr
    or eax, ((1 << 0) | (1 << 11))
    wrmsr

    ; Configure STAR MSR (0xC0000081) selector bases
    ; Kernel CS/SS: 0x0008, User CS/SS base: 0x0018
    mov ecx, 0xC0000081
    rdmsr
    mov edx, 0x00180008
    wrmsr

    ; Configure LSTAR MSR (0xC0000082) target RIP for `syscall` instruction
    mov ecx, 0xC0000082
    mov rax, syscall_handler_asm
    mov rdx, rax
    shr rdx, 32
    wrmsr

    ; Configure SFMASK MSR (0xC0000084) to mask IF and TF on system call entry
    mov ecx, 0xC0000084
    rdmsr
    mov eax, 0x00000300
    wrmsr
    ret

; syscall_handler_asm - Direct entry handler for 64-bit `syscall` instructions
syscall_handler_asm:
    mov [rel user_rsp_temp], rsp
    mov rsp, [rel kernel_stack_temp]

    push qword [rel user_rsp_temp]
    push r11
    push rcx
    push rbp
    push rbx
    push r12
    push r13
    push r14
    push r15

    push rdx
    push rsi
    push rdi
    push r8
    push r9
    push r10

    mov rcx, rdx
    mov rdx, rsi
    mov rsi, rdi
    mov rdi, rax

    call syscall_dispatcher

    cmp eax, 0xDEADBEEF
    je .exit_user_mode

    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rdx

    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    pop rcx
    pop r11
    pop qword [rel user_rsp_temp]

    mov rsp, [rel user_rsp_temp]
    o64 sysret

.exit_user_mode:
global abort_user_mode
abort_user_mode:
    mov rsp, [rel kernel_stack_temp]
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

; jump_to_user - Lower execution privilege level from Ring 0 to Ring 3
; RDI: User mode instruction entry point address (RIP)
; RSI: User mode stack top address (RSP)
jump_to_user:
    push rbp
    push rbx
    push r12
    push r13
    push r14
    push r15

    mov [rel kernel_stack_temp], rsp
    cli

    push 0x23
    push rsi
    push 0x202
    push 0x2B
    push rdi

    xor rax, rax
    xor rbx, rbx
    xor rcx, rcx
    xor rdx, rdx
    xor rsi, rsi
    xor rdi, rdi
    xor rbp, rbp
    xor r8, r8
    xor r9, r9
    xor r10, r10
    xor r11, r11
    xor r12, r12
    xor r13, r13
    xor r14, r14
    xor r15, r15

    iretq
