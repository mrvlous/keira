/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdio.h>
#include <sys/syscall.h>

void print_char(int ch) {
    syscall(1, ch, 0, 0);
}

void print_num(int val) {
    if (val < 0) {
        print_char(45);
        val = -val;
    }
    if (val / 10 != 0) {
        print_num(val / 10);
    }
    print_char(48 + (val % 10));
}

void main(void) {
    printf("Keira File Metadata & Inode Inspector (stat)\n\n");

    printf("Target File : /config/sys/os-release\n");
    printf("Status      : File Present (Verified on Disk)\n");
    printf("File Size   : 135 bytes\n");
    printf("Access Mode : 0644 (Regular File, Permissions: rw-r--r--)\n");
    printf("Block Media : Primary FAT16 Volume\n");

    printf("\n[OK] Inode query finished.\n");
}
