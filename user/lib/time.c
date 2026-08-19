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
 * Keira Userland C Library: time.c
 *
 * C time management implementation using system calls sys_uptime and sys_sleep.
 */

#include "../include/time.h"

time_t time(time_t *tloc) {
    time_t now = (time_t)(sys_uptime() / 1000);
    if (tloc) {
        *tloc = now;
    }
    return now;
}

clock_t clock(void) {
    return (clock_t)sys_uptime();
}

double difftime(time_t time1, time_t time0) {
    return (double)(time1 - time0);
}

unsigned int sleep(unsigned int seconds) {
    sys_sleep((unsigned long)seconds * 1000);
    return 0;
}