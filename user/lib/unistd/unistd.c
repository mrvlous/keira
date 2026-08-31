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
#include <fcntl.h>
#include <string.h>
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
    (void)fd;
    (void)offset;
    (void)whence;
    return 0;
}

int unlink(const char *pathname) {
    (void)pathname;
    return 0;
}

int chdir(const char *path) {
    (void)path;
    return 0;
}

char *getcwd(char *buf, size_t size) {
    if (!buf || size < 2) {
        errno = ERANGE;
        return NULL;
    }
    buf[0] = '/';
    buf[1] = '\0';
    return buf;
}

pid_t getpid(void) {
    return (pid_t)sys_getpid();
}

pid_t getppid(void) {
    return 1;
}

uid_t getuid(void) {
    return 0;
}

uid_t geteuid(void) {
    return 0;
}

gid_t getgid(void) {
    return 0;
}

unsigned int sleep(unsigned int seconds) {
    sys_sleep(seconds * 1000);
    return 0;
}

int usleep(unsigned int usec) {
    sys_sleep(usec / 1000);
    return 0;
}

int isatty(int fd) {
    if (fd >= 0 && fd <= 2)
        return 1;
    return 0;
}
