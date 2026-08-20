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
 * Configures the execution environment, segment organization, and load addresses
 * for standalone Keira user-space applications.
 *
 * Virtual Base: 1 GiB (0x40000000)
 * Section Layout: .text -> .data -> .bss
 */

ENTRY(_start)

SECTIONS {
    . = 0x40000000;

    .text : {
        *(.text)
        *(.text.*)
        *(.rodata)
        *(.rodata.*)
    }

    .data : {
        *(.data)
        *(.data.*)
    }

    .bss : {
        *(.bss)
        *(.bss.*)
        *(COMMON)
    }
}
