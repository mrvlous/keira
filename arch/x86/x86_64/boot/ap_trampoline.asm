; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Application Processor (AP) Real-Mode to Long-Mode Trampoline (x86_64)
;
; Architectural Role:
;   This assembly module implements the full multi-stage bootstrap pipeline for
;   secondary CPU cores (Application Processors) on x86_64 SMP systems.
;
; Execution Lifecycle:
;   1. Cold Start: The Bootstrap Processor (BSP) issues an INIT-SIPI-SIPI sequence
;      targeting physical address 0x8000 (page frame 8).
;   2. Stage 1 (16-Bit Real Mode):
;      Clear interrupts and direction flag; initialize segment registers.
;      Load temporary 32-bit GDT and set CR0.PE (Protected Mode Enable).
;      Far jump to 32-bit code.
;   3. Stage 2 (32-Bit Protected Mode):
;      Reload segment registers with 32-bit data selector.
;      Load CR3 with PML4 page table root address provided by BSP (ap_cr3_val).
;      Enable CR4.PAE (Physical Address Extension bit 5).
;      Enable Long Mode (LME) and No-Execute (NXE) in IA32_EFER MSR (0xC0000080).
;      Enable Paging (CR0.PG) to engage IA-32e Long Mode.
;      Load temporary 64-bit GDT and far jump to 64-bit code segment.
;   4. Stage 3 (64-Bit Long Mode):
;      Establish per-core 64-bit kernel execution stack (ap_stack_val).
;      Pass unique logical core ID in RDI per System V AMD64 ABI convention.
;      Transfer control directly into the Rust secondary entrypoint (ap_main).

%include "constants.inc"

; Physical page address where BSP relocates this trampoline
%define AP_TRAMPOLINE_BASE 0x8000

section .rodata
global ap_trampoline_start
global ap_trampoline_end
global ap_cr3_val
global ap_stack_val
global ap_entry_val
global ap_core_id
global ap_status_flag

ap_trampoline_start:
; Stage 1: 16-Bit Real Mode Entry
[bits 16]
ap_real16_entry:
    cli                         ; Inhibit maskable interrupts
    cld                         ; Clear direction flag

    ; Reset segment registers to linear base 0
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00              ; Temporary real-mode stack

    ; Load temporary 32-bit GDT descriptor
    lgdt [AP_TRAMPOLINE_BASE + (ap_gdt_desc - ap_trampoline_start)]

    ; Set Protection Enable bit in CR0
    mov eax, cr0
    or al, 1
    mov cr0, eax

    ; Far jump to serialize CPU and enter 32-bit Protected Mode
    jmp dword 0x08:(AP_TRAMPOLINE_BASE + (ap_pm32_entry - ap_trampoline_start))

; Stage 2: 32-Bit Protected Mode Entry
[bits 32]
ap_pm32_entry:
    ; Reload data segment selectors with 32-bit data descriptor (0x10)
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax
    mov esp, 0x7C00

    ; Load CR3 with PML4 page table root passed by BSP
    mov eax, [AP_TRAMPOLINE_BASE + (ap_cr3_val - ap_trampoline_start)]
    mov cr3, eax

    ; Enable Physical Address Extension (PAE) in CR4
    mov eax, cr4
    or eax, CR4_PAE_BIT
    mov cr4, eax

    ; Enable Long Mode (LME) and No-Execute (NXE) in IA32_EFER MSR (0xC0000080)
    mov ecx, EFER_MSR
    rdmsr
    or eax, (EFER_LONG_MODE_BIT | EFER_NXE_BIT)
    wrmsr

    ; Enable Paging (CR0.PG) to activate IA-32e Long Mode
    mov eax, cr0
    or eax, CR0_PAGING_BIT
    mov cr0, eax

    ; Load temporary 64-bit GDT descriptor
    lgdt [AP_TRAMPOLINE_BASE + (ap_gdt64_desc - ap_trampoline_start)]

    ; Far jump into 64-bit Long Mode code segment (Selector 0x08)
    jmp dword GDT_CODE64_SEL:(AP_TRAMPOLINE_BASE + (ap_lm64_entry - ap_trampoline_start))

; Stage 3: 64-Bit Long Mode Entry
[bits 64]
ap_lm64_entry:
    ; Reload segment registers with 64-bit data segment selector
    mov ax, GDT_DATA64_SEL
    mov ds, ax
    mov es, ax
    mov ss, ax
    xor ax, ax
    mov fs, ax
    mov gs, ax

    ; Establish dedicated per-core kernel stack pointer
    mov rsp, [abs AP_TRAMPOLINE_BASE + (ap_stack_val - ap_trampoline_start)]

    ; Pass core_id in RDI (1st argument per System V AMD64 ABI):
    ;   extern "C" fn ap_main(core_id: usize) -> !;
    mov rdi, [abs AP_TRAMPOLINE_BASE + (ap_core_id - ap_trampoline_start)]

    ; Call the target Rust entrypoint address configured by BSP
    mov rax, [abs AP_TRAMPOLINE_BASE + (ap_entry_val - ap_trampoline_start)]
    call rax

    ; Halt fallback if secondary core returns
.ap_halt:
    cli
    hlt
    jmp .ap_halt

; Trampoline Parameter Block (Populated dynamically by BSP prior to SIPI)
align 8
ap_cr3_val:     dq 0            ; 64-bit physical address of PML4 page table root
ap_stack_val:   dq 0            ; Top of pre-allocated per-core 64-bit kernel stack
ap_entry_val:   dq 0            ; Function pointer to Rust ap_main entrypoint
ap_core_id:     dq 0            ; Logical core index (0, 1, 2, ...)
ap_status_flag: dq 0            ; Synchronization / handshake status flag

; Temporary 32-Bit GDT (Stage 1 to Stage 2)
align 16
ap_gdt_start:
    dq 0x0000000000000000       ; Null Descriptor
    dq 0x00CF9A000000FFFF       ; 32-bit Code (DPL 0, Base 0, Limit 4GB)
    dq 0x00CF92000000FFFF       ; 32-bit Data (DPL 0, Base 0, Limit 4GB)
ap_gdt_end:

ap_gdt_desc:
    dw ap_gdt_end - ap_gdt_start - 1
    dd AP_TRAMPOLINE_BASE + (ap_gdt_start - ap_trampoline_start)

; Temporary 64-Bit GDT (Stage 2 to Stage 3)
align 16
ap_gdt64_start:
    dq 0x0000000000000000       ; Null Descriptor
    dq 0x00209A0000000000       ; 64-bit Kernel Code (DPL 0, CS.L = 1)
    dq 0x0000920000000000       ; 64-bit Kernel Data (DPL 0, Writable)
ap_gdt64_end:

ap_gdt64_desc:
    dw ap_gdt64_end - ap_gdt64_start - 1
    dd AP_TRAMPOLINE_BASE + (ap_gdt64_start - ap_trampoline_start)

align 16
ap_trampoline_end:
