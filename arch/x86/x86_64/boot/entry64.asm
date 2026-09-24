; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; 64-Bit Long Mode Execution Trampoline (x86_64)
;
; Architectural Role:
;   This assembly module executes immediately following the 64-bit far jump
;   from entry32.asm. It is the first code running in pure 64-bit IA-32e mode.
;
; Hardware Environment at Entry:
;   Processor: 64-bit Long Mode active (CS.L = 1, CS.D = 0).
;   Paging: 4-level paging active with 1 GiB identity map loaded into CR3.
;   Interrupts: Disabled (EFLAGS.IF = 0).
;   Addressability: Full 64-bit general-purpose registers (RAX - R15) available.
;
; Responsibilities:
;   1. Retrieve preserved 32-bit Multiboot2 pointer from the bootstrap stack.
;   2. Reload segment registers with 64-bit data descriptors (DS, ES, SS = 0x10; FS, GS = 0).
;   3. Re-align RSP to the 64-bit kernel bootstrap stack.
;   4. Zero-clear the entire BSS (.bss) section to satisfy C/Rust runtime invariants.
;   5. Place Multiboot2 information pointer into RDI (arg 1 per System V AMD64 ABI).
;   6. Jump directly to Keira top-level Rust initialization routine (kernel_main).

%include "constants.inc"

extern kernel_main
extern __bss_start
extern __bss_end
extern stack_top

section .text
bits 64
global _start64

_start64:
    ; Step 1: Recover Multiboot2 Information Pointer
    ; During entry32.asm, EBX was pushed onto the top of the stack.
    ; Read preserved 32-bit physical address into R12 (zero-extended to 64-bit).
    mov r12d, dword [rel stack_top - 4]

    ; Step 2: Reload 64-Bit Segment Registers
    ; In 64-bit mode, segmentation is largely disabled (base is forced to 0 for
    ; CS, DS, ES, SS), but segment selectors must still point to valid GDT descriptors.
    mov ax, GDT_DATA64_SEL      ; Selector 0x10: 64-bit Kernel Data
    mov ds, ax
    mov es, ax
    mov ss, ax

    ; FS and GS base addresses will be configured via MSRs (IA32_FS_BASE, IA32_GS_BASE)
    ; later for per-CPU and thread-local data structures. Clear selector values now.
    xor ax, ax
    mov fs, ax
    mov gs, ax

    ; Step 3: Establish 64-Bit Kernel Stack Pointer
    ; Align RSP to 32 KiB bootstrap stack. System V AMD64 ABI requires
    ; that RSP + 8 is a multiple of 16 prior to any function call instruction.
    mov rsp, stack_top

    ; Step 4: Zero-Initialize Kernel BSS Section
    ; Clears all uninitialized global and static variables defined in Rust/C crates.
    ; RDI = Destination linear address (__bss_start)
    ; RCX = Number of bytes to zero (__bss_end - __bss_start)
    ; AL  = Fill byte (0)
    mov rdi, __bss_start
    mov rcx, __bss_end
    sub rcx, rdi
    xor al, al
    rep stosb

    ; Step 5: Transfer Control to Rust Kernel (System V AMD64 ABI)
    ; Calling Convention:
    ;   extern "C" fn kernel_main(multiboot_info_ptr: usize) -> !;
    ;   Arg 1: RDI = 64-bit pointer to Multiboot2 Information Structure.
    mov rdi, r12
    call kernel_main

    ; Terminal Fallback Loop
.hang:
    cli
    hlt
    jmp .hang
