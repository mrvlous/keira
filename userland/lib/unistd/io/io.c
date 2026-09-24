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
#include <fcntl.h>
#include <syscall.h>
#include <unistd.h>

ssize_t read(int fd, void *buf, size_t count) {
    int ret = sys_read(fd, (char *)buf, (int)count);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return ret;
}

ssize_t write(int fd, const void *buf, size_t count) {
    int ret = sys_write(fd, (const char *)buf, (int)count);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return ret;
}

int open(const char *pathname, int flags, mode_t mode) {
    int ret = sys_open(pathname, flags, (int)mode);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return ret;
}

int close(int fd) {
    int ret = sys_close(fd);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}

off_t lseek(int fd, off_t offset, int whence) {
    off_t ret = sys_lseek(fd, offset, whence);
    if (ret < 0) {
        errno = (int)-ret;
        return -1;
    }
    return ret;
}

int dup(int oldfd) {
    int ret = sys_dup(oldfd);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return ret;
}

int dup2(int oldfd, int newfd) {
    int ret = sys_dup2(oldfd, newfd);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return ret;
}

int sync(void) {
    int ret = sys_sync();
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}

int fsync(int fd) {
    int ret = sys_fsync(fd);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}

int isatty(int fd) {
    if (fd >= 0 && fd <= 2)
        return 1;
    return 0;
}
