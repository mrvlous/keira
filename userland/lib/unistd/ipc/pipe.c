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
