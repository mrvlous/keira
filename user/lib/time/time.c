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
#include <syscall.h>
#include <time.h>

static struct tm static_tm;
static char asctime_buf[32];

time_t time(time_t *tloc) {
    time_t now = sys_uptime();
    if (tloc) {
        *tloc = now;
    }
    return now;
}

struct tm *gmtime(const time_t *timep) {
    if (!timep)
        return NULL;
    time_t t = *timep;

    static_tm.tm_sec = t % 60;
    t /= 60;
    static_tm.tm_min = t % 60;
    t /= 60;
    static_tm.tm_hour = t % 24;
    t /= 24;

    static_tm.tm_wday = (t + 4) % 7;
    static_tm.tm_year = 126;
    static_tm.tm_mon = 7;
    static_tm.tm_mday = 31;
    static_tm.tm_isdst = 0;

    return &static_tm;
}

char *asctime(const struct tm *tm) {
    if (!tm)
        return NULL;
    snprintf(asctime_buf, sizeof(asctime_buf), "%04d-%02d-%02d %02d:%02d:%02d UTC",
             tm->tm_year + 1900, tm->tm_mon + 1, tm->tm_mday, tm->tm_hour, tm->tm_min, tm->tm_sec);
    return asctime_buf;
}

size_t strftime(char *s, size_t max, const char *format, const struct tm *tm) {
    (void)format;
    if (!s || !tm || max < 20)
        return 0;
    int len = snprintf(s, max, "%04d-%02d-%02d %02d:%02d:%02d", tm->tm_year + 1900, tm->tm_mon + 1,
                       tm->tm_mday, tm->tm_hour, tm->tm_min, tm->tm_sec);
    if (len > 0)
        return (size_t)len;
    return 0;
}

time_t difftime(time_t time1, time_t time0) {
    return (time1 - time0);
}
