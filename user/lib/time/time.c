/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <syscall.h>
#include <time.h>

time_t time(time_t *tloc) {
    time_t t = sys_uptime() / 1000;
    if (tloc)
        *tloc = t;
    return t;
}

static struct tm g_tm;
static char g_asctime_buf[32];

struct tm *gmtime(const time_t *timep) {
    if (!timep)
        return 0;
    time_t t = *timep;
    g_tm.tm_sec = t % 60;
    t /= 60;
    g_tm.tm_min = t % 60;
    t /= 60;
    g_tm.tm_hour = t % 24;
    t /= 24;
    g_tm.tm_mday = (t % 30) + 1;
    g_tm.tm_mon = (t / 30) % 12;
    g_tm.tm_year = 2026 - 1900;
    return &g_tm;
}

char *asctime(const struct tm *tm) {
    if (!tm)
        return 0;
    char *buf = g_asctime_buf;
    buf[0] = 'U';
    buf[1] = 'T';
    buf[2] = 'C';
    buf[3] = ' ';
    buf[4] = '\0';
    return buf;
}
