/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

/*
 * Linker Script Specification for x86_64 Kernel Target
 *
 * Controls kernel memory topology, section placements, alignment constraints,
 * and exported boundary symbols.
 *
 * Physical Base: 1 MiB (0x100000)
 * Section Layout: .multiboot_header -> .text -> .rodata -> .data -> .bss -> .page_tables
 */

ENTRY(_start)

SECTIONS
{
    . = 1M;

    .multiboot_header : ALIGN(8)
    {
        KEEP(*(.multiboot_header))
    }

    .text : ALIGN(4096)
    {
        *(.text)
        *(.text.*)
    }

    .rodata : ALIGN(4096)
    {
        *(.rodata)
        *(.rodata.*)
    }

    .data : ALIGN(4096)
    {
        *(.data)
        *(.data.*)
    }

    .bss : ALIGN(4096)
    {
        __bss_start = .;
        *(.bss)
        *(.bss.*)
        *(COMMON)
        __bss_end = .;
    }

    .page_tables : ALIGN(4096)
    {
        *(.page_tables)
    }

    . = ALIGN(4096);
    __heap_start = .;
    . += 1M;
    __heap_end = .;

    /DISCARD/ :
    {
        *(.comment)
        *(.note)
        *(.note.*)
        *(.eh_frame)
        *(.eh_frame_hdr)
    }
}
