/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

/**
 * Keira User Space: cat utility
 *
 * Concatenate and print files to standard output using VFS system calls.
 */

#include <stdio.h>
#include <string.h>
#include <syscall.h>

/**
 * _start - Entry point for cat user-space utility.
 */
void _start(void) {
    const char *target = "/config/boot/boot.cfg";
    printf("Keira User-Space 'cat' Utility (PID %d)\n", sys_getpid());
    printf("Reading file: %s\n\n", target);

    int fd = sys_open(target, 0);
    if (fd < 0) {
        printf("Error: Failed to open %s\n", target);
        sys_exit();
        return;
    }

    char buffer[256];
    memset(buffer, 0, sizeof(buffer));
    int bytes = sys_read(fd, buffer, sizeof(buffer) - 1);
    if (bytes > 0) {
        printf("=== %s (%d bytes) ===\n", target, bytes);
        printf("%s\n", buffer);
    } else {
        printf("File is empty or unreadable.\n");
    }

    sys_close(fd);
    sys_exit();
}