; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Operating System Kernel
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Page Table Initialization and Early Identity Mapping
;
; Configures 4-level paging structures (PML4 -> PDPT -> PD) to identity-map
; 1 GiB of physical memory using 2 MiB huge pages during early bootstrap.
;
; Section .page_tables is allocated in a non-BSS custom section to avoid
; accidental erasure during early zero-initialization phases.

%include "constants.inc"

section .page_tables write nobits
align 4096

global pml4_table
pml4_table:
    resb 4096

global pdpt_table
pdpt_table:
    resb 4096

global pd_table
pd_table:
    resb 4096

section .text
bits 32

global setup_page_tables
setup_page_tables:
    ; Step 1: PML4[0] -> PDPT Table Base Address | PRESENT | WRITABLE
    mov eax, pdpt_table
    or  eax, PAGE_RW_PRESENT
    mov [pml4_table], eax

    ; Step 2: PDPT[0] -> PD Table Base Address | PRESENT | WRITABLE
    mov eax, pd_table
    or  eax, PAGE_RW_PRESENT
    mov [pdpt_table], eax

    ; Step 3: Populate 512 entries of 2 MiB pages in PD table (1 GiB total identity mapping)
    mov ecx, 0
.map_loop:
    mov eax, ecx
    shl eax, 21
    or  eax, PAGE_PRESENT | PAGE_WRITABLE | PAGE_HUGE
    mov [pd_table + ecx * 8], eax
    mov dword [pd_table + ecx * 8 + 4], 0
    inc ecx
    cmp ecx, 512
    jne .map_loop

    ret
    