; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Fast System Call Infrastructure and User Mode Transitions (x86_64)
;
; Architectural Role:
;   This assembly module implements hardware fast system call entry and exit
;   mechanisms (syscall / sysretq), per-CPU GS-based stack switching (swapgs),
;   and Ring 0 to Ring 3 privilege transitions (jump_to_user).
;
; Hardware Fast System Call Mechanics (syscall / sysretq):
;   The x86_64 syscall instruction bypasses the traditional Interrupt Descriptor
;   Table (IDT) to provide high-speed, sub-microsecond privilege escalation:
;   1. Hardware automatically copies RIP to RCX and RFLAGS to R11.
;   2. Hardware masks RFLAGS with IA32_FMASK (disabling interrupts).
;   3. Hardware loads CS from IA32_STAR[47:32] and SS from IA32_STAR[47:32] + 8.
;   4. Hardware transfers execution directly to the address in IA32_LSTAR.
;
; Per-CPU Data Area Offsets (Accessed via GS segment after swapgs):
;   [gs:0x08]: Saved Userland Stack Pointer (User RSP)
;   [gs:0x10]: Kernel Syscall Execution Stack Top (Kernel RSP0)
;   [gs:0x18]: Saved Kernel Return Stack for jump_to_user recovery
;   [gs:0x28 - 0x68]: Callee-saved register scratchpad
;
; Calling Conventions:
;   Userland System Call Registers:
;     RAX: System call number
;     RDI: Argument 1
;     RSI: Argument 2
;     RDX: Argument 3
;     R10: Argument 4 (note: RCX is overwritten by hardware with user RIP)
;     R8:  Argument 5
;     R9:  Argument 6
;     Return Value: RAX
;
;   Rust Dispatcher Signature (System V AMD64 ABI):
;     syscall_dispatcher(n: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, a6: u64)
;     RDI = n  (from RAX)
;     RSI = a1 (from user RDI)
;     RDX = a2 (from user RSI)
;     RCX = a3 (from user RDX)
;     R8  = a4 (from user R10)
;     R9  = a5 (from user R8)
;     [rsp] = a6 (from user R9, 7th argument passed via stack)

global init_syscall_msrs
global syscall_handler_asm
global jump_to_user
global abort_user_mode
global main_kernel_stack

extern syscall_dispatcher

section .data
align 8
main_kernel_stack: dq 0

section .text
bits 64

; init_syscall_msrs: Configure CPU Model-Specific Registers for Fast Syscalls
; Configures MSRs governing syscall and sysretq instructions.
; Must be executed on every CPU core (BSP and all APs) during initialization.
init_syscall_msrs:
    ; 1. Enable System Call Extensions (SCE) in IA32_EFER (0xC0000080)
    mov ecx, 0xC0000080
    rdmsr
    or eax, ((1 << 0) | (1 << 11)) ; Bit 0: SCE, Bit 11: NXE
    wrmsr

    ; 2. Configure Segment Selectors in IA32_STAR (0xC0000081)
    ; Bits 47:32 -> Kernel CS = 0x0008, Kernel SS = 0x0010
    ; Bits 63:48 -> User CS/SS base selector = 0x0018 (sysret uses 0x18+16 and 0x18+8)
    mov ecx, 0xC0000081
    rdmsr
    mov edx, 0x00180008
    wrmsr

    ; 3. Configure Entrypoint Address in IA32_LSTAR (0xC0000082)
    ; Specifies the linear target address for the syscall instruction.
    mov ecx, 0xC0000082
    mov rax, syscall_handler_asm
    mov rdx, rax
    shr rdx, 32                 ; High 32 bits into EDX
    wrmsr

    ; 4. Configure RFLAGS Mask in IA32_SFMASK (0xC0000084)
    ; Masks Bit 9 (IF - Interrupt Flag) and Bit 8 (TF - Trap Flag).
    ; Guarantees interrupts are disabled immediately upon entering the kernel.
    mov ecx, 0xC0000084
    rdmsr
    mov eax, 0x00000300         ; IF (0x200) | TF (0x100)
    wrmsr
    ret

; syscall_handler_asm: Direct Entrypoint for 64-Bit syscall Instruction
syscall_handler_asm:
    ; Swap GS register with IA32_KERNEL_GS_BASE to access per-CPU data
    swapgs

    ; Save user stack pointer and load dedicated per-CPU kernel syscall stack
    mov [gs:0x08], rsp
    mov rsp, [gs:0x10]

    ; Preserve User Context on Kernel Stack
    push qword [gs:0x08]        ; User RSP
    push r11                    ; User RFLAGS (saved by hardware in R11)
    push rcx                    ; User RIP (saved by hardware in RCX)
    push rbp
    push rbx
    push r12
    push r13
    push r14
    push r15

    ; Mirror state to GS scratchpad for fault diagnostic reporting
    mov [gs:0x28], rcx
    mov [gs:0x30], r11
    mov [gs:0x38], rbx
    mov [gs:0x40], rbp
    mov [gs:0x48], r12
    mov [gs:0x50], r13
    mov [gs:0x58], r14
    mov r15, [rsp]
    mov [gs:0x60], r15
    mov r15, [gs:0x08]
    mov [gs:0x68], r15

    ; Save argument registers across dispatch
    push rdx
    push rsi
    push rdi
    push r8
    push r9
    push r10

    ; Remap Registers for System V AMD64 ABI Call
    ; System V ABI: RDI, RSI, RDX, RCX, R8, R9, stack (16-byte aligned)
    push 0                      ; Padding to maintain 16-byte stack alignment
    push r9                     ; Argument 6 (user R9) passed on stack

    mov r9, r8                  ; Arg 5 = user R8
    mov r8, r10                 ; Arg 4 = user R10 (was passed in R10 by userland)
    mov rcx, rdx                ; Arg 3 = user RDX
    mov rdx, rsi                ; Arg 2 = user RSI
    mov rsi, rdi                ; Arg 1 = user RDI
    mov rdi, rax                ; Syscall Number = user RAX

    call syscall_dispatcher     ; Execute Rust syscall handler (return value in RAX)
    add rsp, 16                 ; Pop 7th argument and padding

    ; Check if task requested termination (sentinel 0xDEADBEEF)
    cmp eax, 0xDEADBEEF
    je .exit_user_mode

    ; Restore preserved user registers
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
    pop rcx                     ; User RIP restored into RCX for sysretq
    pop r11                     ; User RFLAGS restored into R11 for sysretq
    pop rsp                     ; Restore User RSP directly

    ; Restore User GS base
    swapgs

    ; Fast return to userland: loads RIP from RCX, RFLAGS from R11, CS/SS from STAR
    o64 sysret

; abort_user_mode: Terminate userland execution and return to kernel caller
.exit_user_mode:
abort_user_mode:
    ; Recover kernel stack captured during jump_to_user
    mov rsp, [gs:0x18]
    test rsp, rsp
    jnz .stack_ok
    mov rsp, [rel main_kernel_stack]
.stack_ok:
    ; Reload kernel data segments
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax

    ; Restore preserved kernel callee-saved registers
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp
    ret

; jump_to_user: Lower execution privilege level from Ring 0 to Ring 3 (64-Bit)
; Calling Convention:
;   System V AMD64 ABI:
;     extern "C" fn jump_to_user(entry_point: u64, user_stack: u64) -> !;
;     RDI: Userland instruction pointer (RIP)
;     RSI: Userland stack pointer (RSP)
jump_to_user:
    ; Preserve kernel callee-saved registers
    push rbp
    push rbx
    push r12
    push r13
    push r14
    push r15

    ; Save kernel stack for exit abort recovery
    mov [gs:0x18], rsp
    mov [rel main_kernel_stack], rsp
    cli

    ; Construct Synthetic iretq Frame
    push 0x23                   ; User SS (Selector 0x20 | RPL 3)
    push rsi                    ; User RSP
    push 0x202                  ; RFLAGS (IF enabled, bit 1 set)
    push 0x2B                   ; User CS (Selector 0x28 | RPL 3)
    push rdi                    ; User RIP

    ; Forward argc and argv into RDI and RSI for userland _start entrypoint
    mov r8, rsi
    mov rdi, [r8]               ; argc = *(user_stack)
    lea rsi, [r8 + 8]           ; argv = user_stack + 8

    ; Scrub general-purpose registers to eliminate kernel data leakage
    xor rax, rax
    xor rbx, rbx
    xor rcx, rcx
    xor rdx, rdx
    xor rbp, rbp
    xor r8, r8
    xor r9, r9
    xor r10, r10
    xor r11, r11
    xor r12, r12
    xor r13, r13
    xor r14, r14
    xor r15, r15

    ; Switch to user GS base before lowering privilege
    swapgs
    iretq                       ; Transition to Ring 3 Userland
