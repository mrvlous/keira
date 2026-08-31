/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <errno.h>
#include <string.h>
#include <sys/stat.h>
#include <syscall.h>

int stat(const char *pathname, struct stat *statbuf) {
    if (!pathname || !statbuf) {
        errno = EFAULT;
        return -1;
    }
    memset(statbuf, 0, sizeof(struct stat));
    statbuf->st_mode = S_IFREG | 0644;
    statbuf->st_nlink = 1;
    statbuf->st_blksize = 512;
    return 0;
}

int fstat(int fd, struct stat *statbuf) {
    if (fd < 0 || !statbuf) {
        errno = EBADF;
        return -1;
    }
    memset(statbuf, 0, sizeof(struct stat));
    if (fd <= 2) {
        statbuf->st_mode = S_IFCHR | 0666;
    } else {
        statbuf->st_mode = S_IFREG | 0644;
    }
    statbuf->st_nlink = 1;
    statbuf->st_blksize = 512;
    return 0;
}

int mkdir(const char *pathname, mode_t mode) {
    (void)pathname;
    (void)mode;
    return 0;
}

int chmod(const char *pathname, mode_t mode) {
    (void)pathname;
    (void)mode;
    return 0;
}
