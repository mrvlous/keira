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
 * Keira User Space: Core Init Process (PID 1)
 *
 * Initial process launched after Ring 3 user-mode initialization.
 * Verifies system calls, file system operations, dynamic heap memory,
 * and process execution.
 */

#include <malloc.h>
#include <stdio.h>
#include <string.h>
#include <syscall.h>

/**
 * _start - Core entry point for the user-space init process.
 *
 * This function initializes Ring 3 execution, tests standard libraries
 * (stdio, string), filesystems, heaps, and other system call wrappers
 * before exiting back to the kernel shell.
 */
void _start(void) {
    printf("Keira User-Space Init Process (PID %d)\n", sys_getpid());
    printf("Running in Ring 3 (User Mode) CPU protection level.\n\n");

    const char *greet = "Hello from Ring 3 User Land!";
    char buffer[64];
    memset(buffer, 0, sizeof(buffer));
    strncpy(buffer, greet, sizeof(buffer) - 1);
    printf("Testing String Copy & Format: '%s' (Length: %d)\n", buffer, (int)strlen(buffer));

    int val_dec = -12345;
    unsigned int val_hex = 0xABCDEF12;
    printf("Testing Decimal Formatting  : %d\n", val_dec);
    printf("Testing Hexadecimal Formatting: %x\n\n", val_hex);

    printf("Testing System Uptime: %d ms\n", (int)sys_uptime());

    printf("Testing File System Subsystem:\n");
    int fd = sys_open("/data/log/test.log", 1);
    if (fd >= 0) {
        printf("  Opened /data/log/test.log in write mode (FD: %d)\n", fd);
        const char *msg = "Keira File System Syscalls Verified!";
        int written = sys_write(fd, msg, (int)strlen(msg));
        printf("  Wrote %d bytes to file.\n", written);

        sys_seek(fd, 0);

        char read_buf[64];
        memset(read_buf, 0, sizeof(read_buf));
        int read_bytes = sys_read(fd, read_buf, sizeof(read_buf) - 1);
        printf("  Read %d bytes back: '%s'\n", read_bytes, read_buf);

        sys_close(fd);
        printf("  Closed file descriptor.\n\n");
    } else {
        printf("  Failed to open /data/log/test.log\n\n");
    }

    printf("Testing Dynamic Memory (malloc & free):\n");
    char *heap_str = (char *)malloc(32);
    if (heap_str != NULL) {
        strncpy(heap_str, "Heap Allocation Verified!", 31);
        printf("  Allocated 32 bytes on heap: '%s'\n", heap_str);
        free(heap_str);
        printf("  Freed heap memory successfully.\n\n");
    } else {
        printf("  Failed to allocate memory on heap.\n\n");
    }

    char cwd[128];
    memset(cwd, 0, sizeof(cwd));
    int cwd_len = sys_getcwd(cwd, sizeof(cwd) - 1);
    if (cwd_len > 0) {
        printf("Current Working Directory: %s\n", cwd);
    }

    printf("Init process checks complete. Returning to Kernel shell.\n");
    sys_exit();
}
