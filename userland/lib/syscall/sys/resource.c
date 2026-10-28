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
#include <sys/resource.h>
#include <sys/syscall.h>
#include <unistd.h>

int sys_getrlimit(int resource, struct rlimit *rlim) {
    int64_t ret = syscall2(SYS_GETRLIMIT, (uint64_t)resource, (uint64_t)(uintptr_t)rlim);
    return (int)ret;
}

int sys_setrlimit(int resource, const struct rlimit *rlim) {
    int64_t ret = syscall2(SYS_SETRLIMIT, (uint64_t)resource, (uint64_t)(uintptr_t)rlim);
    return (int)ret;
}

int sys_prlimit64(pid_t pid, int resource, const struct rlimit *new_limit,
                  struct rlimit *old_limit) {
    int64_t ret = syscall4(SYS_PRLIMIT64, (uint64_t)pid, (uint64_t)resource,
                           (uint64_t)(uintptr_t)new_limit, (uint64_t)(uintptr_t)old_limit);
    return (int)ret;
}

int getrlimit(int resource, struct rlimit *rlim) {
    int ret = sys_getrlimit(resource, rlim);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}

int setrlimit(int resource, const struct rlimit *rlim) {
    int ret = sys_setrlimit(resource, rlim);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}

int prlimit(pid_t pid, int resource, const struct rlimit *new_limit, struct rlimit *old_limit) {
    int ret = sys_prlimit64(pid, resource, new_limit, old_limit);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}
