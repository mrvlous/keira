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
#include <stddef.h>
#include <syscall.h>
#include <unistd.h>

int unlink(const char *pathname) {
    (void)pathname;
    return 0;
}

int chdir(const char *path) {
    int ret = sys_chdir(path);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}

char *getcwd(char *buf, size_t size) {
    if (!buf || size == 0) {
        errno = EINVAL;
        return NULL;
    }
    ssize_t ret = sys_getcwd(buf, size);
    if (ret < 0) {
        errno = (int)-ret;
        return NULL;
    }
    return buf;
}
