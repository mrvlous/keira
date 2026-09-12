; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Low-Level System Call Trampolines and User Mode Privilege Transitions (i686 32-Bit)

global jump_to_user
global user_rsp_temp
global kernel_stack_temp

extern syscall_dispatcher

section .data
align 4
user_rsp_temp:     dd 0
kernel_stack_temp: dd 0

section .text
bits 32

; jump_to_user - Lower execution privilege level from Ring 0 to Ring 3 (32-bit)
; [esp+4] = entry_point (EIP)
; [esp+8] = user_stack (ESP)
jump_to_user:
    push ebp
    push ebx
    push esi
    push edi

    mov [kernel_stack_temp], esp
    cli

    mov edx, [esp + 20]   ; entry_point (low 32-bit of u64)
    mov ecx, [esp + 28]   ; user_stack (low 32-bit of u64)

    ; Set User Data Segment (0x20 | 3 = 0x23)
    mov ax, 0x23
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; Push iretd frame: SS, ESP, EFLAGS, CS, EIP
    push dword 0x23       ; User SS
    push ecx              ; User ESP
    push dword 0x202      ; EFLAGS (IF enabled)
    push dword 0x1B       ; User CS (0x18 | 3 = 0x1B)
    push edx              ; User EIP

    xor eax, eax
    xor ebx, ebx
    xor ecx, ecx
    xor edx, edx
    xor esi, esi
    xor edi, edi
    xor ebp, ebp

    iretd

global isr128
isr128:
    push ebp
    push edi
    push esi
    push edx
    push ecx
    push ebx

    push eax

    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    pop eax

    push dword 0
    push edx
    push dword 0
    push ecx
    push dword 0
    push ebx
    push dword 0
    push eax

    call syscall_dispatcher
    add esp, 32

    cmp eax, 0xDEADBEEF
    je .exit_user_mode

    mov dx, 0x23
    mov ds, dx
    mov es, dx
    mov fs, dx
    mov gs, dx

    pop ebx
    pop ecx
    pop edx
    pop esi
    pop edi
    pop ebp
    iretd

.exit_user_mode:
global abort_user_mode
abort_user_mode:
    mov esp, [kernel_stack_temp]
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    pop edi
    pop esi
    pop ebx
    pop ebp
    ret
