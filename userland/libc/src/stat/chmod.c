// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
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
    if (!pathname) {
        errno = EINVAL;
        return -1;
    }
    int ret = sys_chmod(pathname, mode);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}
