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
 * Keira User Space: uptime utility
 *
 * Display system uptime measured from hardware CMOS RTC and timer ticks.
 */

#include <stdio.h>
#include <syscall.h>

/**
 * _start - Entry point for uptime user-space program.
 */
void _start(void) {
    unsigned long ms = sys_uptime();
    unsigned long total_sec = ms / 1000;
    unsigned int days = (unsigned int)(total_sec / 86400);
    unsigned int hours = (unsigned int)((total_sec % 86400) / 3600);
    unsigned int minutes = (unsigned int)((total_sec % 3600) / 60);
    unsigned int seconds = (unsigned int)(total_sec % 60);

    printf("Keira Kernel Uptime (PID %d):\n", sys_getpid());
    printf("  %d days, %d hours, %d minutes, %d seconds (%d ms total)\n", (int)days, (int)hours,
           (int)minutes, (int)seconds, (int)ms);

    sys_exit();
}
