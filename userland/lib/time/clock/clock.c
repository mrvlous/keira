/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdint.h>
#include <sys/syscall.h>
#include <syscall.h>
#include <time.h>

time_t time(time_t *tloc) {
    time_t now = sys_uptime();
    if (tloc) {
        *tloc = now;
    }
    return now;
}

time_t difftime(time_t time1, time_t time0) {
    return (time1 - time0);
}

int clock_gettime(clockid_t clk_id, struct timespec *tp) {
    if (!tp)
        return -1;
    return (int)syscall2(SYS_CLOCK_GETTIME, (uint64_t)clk_id, (uint64_t)(uintptr_t)tp);
}

int nanosleep(const struct timespec *req, struct timespec *rem) {
    if (!req)
        return -1;
    return (int)syscall2(SYS_NANOSLEEP, (uint64_t)(uintptr_t)req, (uint64_t)(uintptr_t)rem);
}
