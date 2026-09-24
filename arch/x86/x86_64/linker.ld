/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

/*
 * Linker Script Specification for x86_64 Kernel Target
 *
 * Architectural Role:
 *   Defines the physical memory topology, section mapping, 4 KiB page-alignment
 *   constraints, and exported boundary symbols for Keira on 64-bit x86_64 hardware.
 *
 * Memory Layout Topology:
 *   Physical Load Base: 1 MiB (0x00100000). Guarantees the kernel binary is
 *   loaded safely above legacy BIOS memory areas and video buffers.
 *   Multiboot2 Header: Placed first in the physical image and aligned to 8 bytes
 *   to guarantee discoverability by Multiboot2 bootloaders within the 32 KiB window.
 *   Text Section (.text): Executable kernel code aligned to 4096-byte page boundaries.
 *   Read-Only Data (.rodata): String constants, jump tables, immutable tables.
 *   Initialized Data (.data): Initialized mutable global variables.
 *   Uninitialized Data (.bss): Zero-initialized variables, cleared during entry64.asm.
 *   Page Tables (.page_tables): Custom section holding initial 4-level paging
 *   structures; kept distinct from .bss so clearing .bss does not destroy active tables.
 *   Early Bootstrap Heap: 1 MiB reserved heap space mapped after .bss.
 *   Discarded Sections: Compiler comment strings, ELF notes, exception unwind tables.
 */

ENTRY(_start)

SECTIONS
{
    /* Physical Load Base: 1 MiB */
    . = 1M;

    /* Multiboot2 Header Section: Must be 8-byte aligned and first in binary */
    .multiboot_header : ALIGN(8)
    {
        KEEP(*(.multiboot_header))
    }

    /* Executable Code Section (.text) */
    __text_start = .;
    .text : ALIGN(4096)
    {
        *(.text)
        *(.text.*)
    }
    __text_end = .;

    /* Read-Only Data Section (.rodata) */
    .rodata : ALIGN(4096)
    {
        *(.rodata)
        *(.rodata.*)
    }

    /* Initialized Read-Write Data Section (.data) */
    .data : ALIGN(4096)
    {
        *(.data)
        *(.data.*)
    }

    /* Uninitialized Data Section (.bss) */
    .bss : ALIGN(4096)
    {
        __bss_start = .;
        *(.bss)
        *(.bss.*)
        *(COMMON)
        __bss_end = .;
    }

    /* Early Architecture 4-Level Page Tables */
    .page_tables : ALIGN(4096)
    {
        *(.page_tables)
    }

    /* Early Kernel Bootstrap Heap Buffer (1 MiB) */
    . = ALIGN(4096);
    __heap_start = .;
    . += 1M;
    __heap_end = .;

    /* Unused Metadata and Unwind Tables Discard */
    /DISCARD/ :
    {
        *(.comment)
        *(.note)
        *(.note.*)
        *(.eh_frame)
        *(.eh_frame_hdr)
    }
}
