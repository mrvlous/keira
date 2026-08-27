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
 * User Space Linker Script for i686 (32-Bit) User Targets
 * Virtual Base: 0x01000000 (16 MiB)
 */

OUTPUT_FORMAT("elf32-i386")
ENTRY(_start)

PHDRS {
    text PT_LOAD FLAGS(5); /* Read + Execute */
    data PT_LOAD FLAGS(6); /* Read + Write */
}

SECTIONS {
    . = 0x01000000;

    .text : {
        *(.text)
        *(.text.*)
        *(.rodata)
        *(.rodata.*)
    } :text

    . = ALIGN(0x1000);

    .data : {
        *(.data)
        *(.data.*)
        *(.got)
        *(.got.plt)
    } :data

    .bss : {
        *(.bss)
        *(.bss.*)
        *(COMMON)
    } :data

    /DISCARD/ : {
        *(.comment)
        *(.note.gnu.build-id)
    }
}
