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
 * Linker Script Specification for Ring 3 User Space Programs
 *
 * Enforces strict W^X separation between executable code (.text/.rodata) and writable data (.data/.bss).
 *
 * Virtual Base: 1 GiB (0x40000000)
 */

ENTRY(_start)

PHDRS {
    text PT_LOAD FLAGS(5); /* PF_R | PF_X (Read + Execute) */
    data PT_LOAD FLAGS(6); /* PF_R | PF_W (Read + Write) */
}

SECTIONS {
    . = 0x40000000;

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
    } :data

    .bss : {
        *(.bss)
        *(.bss.*)
        *(COMMON)
    } :data
}
