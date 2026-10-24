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
#include <string.h>
#include <sys/stat.h>
#include <sys/syscall.h>

int stat(const char *pathname, struct stat *statbuf) {
    if (!pathname || !statbuf) {
        errno = EFAULT;
        return -1;
    }
    int ret = sys_stat(pathname, statbuf);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}

int fstat(int fd, struct stat *statbuf) {
    if (fd < 0 || !statbuf) {
        errno = EBADF;
        return -1;
    }
    int ret = sys_fstat(fd, statbuf);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}
