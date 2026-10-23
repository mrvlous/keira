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
#include <sys/syscall.h>
#include <unistd.h>

int chown(const char *pathname, uid_t owner, gid_t group) {
    if (!pathname) {
        errno = EINVAL;
        return -1;
    }
    int ret = sys_chown(pathname, owner, group);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}
