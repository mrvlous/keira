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
 * Linker Script Specification for i686 (32-Bit) Kernel Target
 *
 * Architectural Role:
 *   Defines the physical memory topology, section layout, 4 KiB page-alignment
 *   constraints, and exported boundary symbols for Keira on 32-bit x86 hardware.
 *
 * Memory Layout Topology:
 *   Load Address: 1 MiB (0x00100000). Avoids real-mode memory holes, BIOS Data
 *   Area (BDA), and memory-mapped video framebuffers below 1 MiB.
 *   Multiboot2 Header: Guaranteed to reside within the first 32 KiB window.
 *   Text Section: Code segments aligned to 4 KiB page boundaries.
 *   Read-Only Data (.rodata): String literals, jump tables, constant structures.
 *   Read-Write Data (.data): Initialized global/static variables.
 *   Block Starting Symbol (.bss): Zero-initialized memory for uninitialized globals.
 *   Early Page Tables (.page_tables): Custom section preserved across BSS wipes.
 *   Early Bootstrap Heap: 1 MiB reserved heap space mapped after .bss.
 *   Discarded Sections: Compiler notes, debug comments, exception unwind headers.
 */

OUTPUT_FORMAT("elf32-i386")
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

    /* Early Architecture Page Tables */
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
