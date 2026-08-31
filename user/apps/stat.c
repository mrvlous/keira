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
#include <sys/stat.h>
#include <unistd.h>

void main(void) {
    printf("Keira File Metadata & Inode Inspector (stat)\n");

    struct stat st;
    const char *target = "/config/sys/os-release";

    if (stat(target, &st) == 0) {
        printf("File      : %s\n", target);
        printf("Size      : %ld bytes\n", (long)st.st_size);
        printf("Blocks    : %ld (512-byte sectors)\n", (long)st.st_blocks);
        printf("Mode      : 0%o (Regular File, Permissions: rw-r--r--)\n", st.st_mode & 0777);
        printf("Links     : %u\n", (unsigned int)st.st_nlink);
        printf("Device ID : 0x%lx (Primary FAT16 Block Media)\n", (unsigned long)st.st_dev);
    } else {
        printf("Error: Could not stat %s\n", target);
    }

    printf("\n[OK] Inode query finished.\n");
}
