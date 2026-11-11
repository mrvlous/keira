/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

/*
 * Linker Script Specification for Ring 3 User Space Programs (x86_64)
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

    /DISCARD/ : {
        *(.comment)
        *(.note.gnu.build-id)
    }
}
