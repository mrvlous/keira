/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdint.h>
#include <sys/syscall.h>
#include <unistd.h>

void sys_print_char(char c) {
    syscall1(SYS_PUTC, (uint64_t)(unsigned char)c);
}

ssize_t sys_read(int fd, void *buf, size_t count) {
    return (ssize_t)syscall3(SYS_READ, (uint64_t)fd, (uint64_t)(uintptr_t)buf, (uint64_t)count);
}

ssize_t sys_write(int fd, const void *buf, size_t count) {
    return (ssize_t)syscall3(SYS_WRITE, (uint64_t)fd, (uint64_t)(uintptr_t)buf, (uint64_t)count);
}

int sys_open(const char *filename, int flags, int mode) {
    (void)mode;
    uint64_t write_flag = (flags != 0) ? 1 : 0;
    return (int)syscall2(SYS_OPEN, (uint64_t)(uintptr_t)filename, write_flag);
}

int sys_close(int fd) {
    return (int)syscall1(SYS_CLOSE, (uint64_t)fd);
}

off_t sys_lseek(int fd, off_t offset, int whence) {
    return (off_t)syscall3(SYS_LSEEK, (uint64_t)fd, (uint64_t)offset, (uint64_t)whence);
}

int sys_chdir(const char *path) {
    return (int)syscall1(SYS_CHDIR, (uint64_t)(uintptr_t)path);
}

ssize_t sys_getcwd(char *buf, size_t size) {
    return (ssize_t)syscall2(SYS_GETCWD, (uint64_t)(uintptr_t)buf, (uint64_t)size);
}

int sys_sync(void) {
    return (int)syscall0(SYS_SYNC);
}

int sys_fsync(int fd) {
    return (int)syscall1(SYS_FSYNC, (uint64_t)fd);
}

int sys_dup(int oldfd) {
    return (int)syscall1(SYS_DUP, (uint64_t)oldfd);
}

int sys_dup2(int oldfd, int newfd) {
    return (int)syscall2(SYS_DUP2, (uint64_t)oldfd, (uint64_t)newfd);
}
