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
#include <sys/stat.h>
#include <sys/syscall.h>

int mkdir(const char *pathname, mode_t mode) {
    if (!pathname) {
        errno = EINVAL;
        return -1;
    }
    int ret = sys_mkdir(pathname, mode);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}

int chmod(const char *pathname, mode_t mode) {
    (void)pathname;
    (void)mode;
    return 0;
}
