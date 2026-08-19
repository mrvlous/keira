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
 * Keira User Space: echo utility
 *
 * Output string arguments to standard output stream.
 */

#include <stdio.h>
#include <syscall.h>

/**
 * _start - Entry point for echo user-space program.
 */
void _start(void) {
    const char *msg = "Keira Kernel Userland Echo Process (PID %d)\n";
    printf(msg, sys_getpid());
    printf("Keira Kernel User-Space Ring 3 Execution Active.\n");
    sys_exit();
}