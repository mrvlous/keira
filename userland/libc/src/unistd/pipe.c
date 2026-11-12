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
#include <stdint.h>
#include <sys/syscall.h>
#include <syscall.h>
#include <unistd.h>

int pipe(int pipefd[2]) {
    if (!pipefd) {
        errno = EFAULT;
        return -1;
    }
    int64_t ret = syscall1(SYS_PIPE, (uint64_t)(uintptr_t)pipefd);
    if (ret < 0) {
        errno = (int)-ret;
        return -1;
    }
    return 0;
}

int pipe2(int pipefd[2], int flags) {
    if (!pipefd) {
        errno = EFAULT;
        return -1;
    }
    int64_t ret = syscall2(SYS_PIPE2, (uint64_t)(uintptr_t)pipefd, (uint64_t)flags);
    if (ret < 0) {
        errno = (int)-ret;
        return -1;
    }
    return 0;
}
