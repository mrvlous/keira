; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Page Table Initialization and Early Identity Mapping (x86_64)
;
; Architectural Role:
;   This assembly module constructs the initial 4-level paging hierarchy
;   required to activate IA-32e Long Mode during 32-bit bootstrap in entry32.asm.
;
; Memory Hierarchy (4-Level Paging Architecture):
;   PML4 Table (Page Map Level 4)
;     Entry 0: Points to PDPT (Page Directory Pointer Table)
;       Entry 0: Points to PD (Page Directory Table)
;         512 x 2 MiB Huge Pages = 1 GiB Identity Mapped
;
; Memory Section Placement (.page_tables):
;   The page table structures are allocated in a dedicated .page_tables section.
;   This section is marked as uninitialized data (nobits) but is segregated from
;   the standard .bss section so that the rep stosb BSS-clearing loop in
;   entry64.asm does not inadvertently destroy the active page tables while the
;   CPU is actively executing via them.

%include "constants.inc"

; Page Table Storage Allocation (Aligned to 4096-Byte Hardware Page Boundaries)
section .page_tables write nobits
align 4096

; Page Map Level 4 (PML4) Table: Root of 4-level paging hierarchy (4096 bytes)
global pml4_table
pml4_table:
    resb 4096

; Page Directory Pointer Table (PDPT): Level 3 translation table (4096 bytes)
global pdpt_table
pdpt_table:
    resb 4096

; Page Directory (PD) Table: Level 2 translation table (4096 bytes)
global pd_table
pd_table:
    resb 4096

; Page Table Setup Routine (Executes in 32-Bit Protected Mode)
section .text
bits 32

global setup_page_tables
setup_page_tables:
    ; Step 1: Link PML4 Entry 0 -> PDPT Base Address
    ; Lower 12 bits of address contain attribute flags:
    ;   PAGE_PRESENT (bit 0 = 1) | PAGE_WRITABLE (bit 1 = 1)
    mov eax, pdpt_table
    or  eax, PAGE_RW_PRESENT
    mov [pml4_table], eax

    ; Step 2: Link PDPT Entry 0 -> PD Base Address
    mov eax, pd_table
    or  eax, PAGE_RW_PRESENT
    mov [pdpt_table], eax

    ; Step 3: Populate 512 Page Directory Entries (2 MiB Huge Pages)
    ; Each 2 MiB huge page covers 0x200000 bytes.
    ; 512 entries * 2 MiB = 1,073,741,824 bytes (1 GiB) of physical RAM.
    ;
    ; Descriptor Bit Configuration:
    ;   Bit 0 (P)  = 1 (Present)
    ;   Bit 1 (RW) = 1 (Read/Write)
    ;   Bit 7 (PS) = 1 (Page Size: 2 MiB huge page)
    ;   Bits 21..31 = Physical frame base address
    mov ecx, 0                  ; Loop counter (entry index 0 to 511)
.map_loop:
    mov eax, ecx
    shl eax, 21                 ; Calculate base address: ecx * 2 MiB (ecx << 21)
    or  eax, PAGE_PRESENT | PAGE_WRITABLE | PAGE_HUGE

    ; Store lower 32 bits (base address and flags)
    mov [pd_table + ecx * 8], eax

    ; Store upper 32 bits (zero for physical addresses < 4 GiB)
    mov dword [pd_table + ecx * 8 + 4], 0

    inc ecx
    cmp ecx, 512
    jne .map_loop

    ret
