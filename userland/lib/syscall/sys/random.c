/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <errno.h>
#include <sys/random.h>
#include <sys/syscall.h>
#include <unistd.h>

ssize_t sys_getrandom(void *buf, size_t buflen, unsigned int flags) {
    int64_t ret =
        syscall3(SYS_GETRANDOM, (uint64_t)(uintptr_t)buf, (uint64_t)buflen, (uint64_t)flags);
    return (ssize_t)ret;
}

ssize_t getrandom(void *buf, size_t buflen, unsigned int flags) {
    ssize_t ret = sys_getrandom(buf, buflen, flags);
    if (ret < 0) {
        errno = (int)(-ret);
        return -1;
    }
    return ret;
}
