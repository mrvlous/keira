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

pid_t getpid(void) {
    return (pid_t)sys_getpid();
}

pid_t getppid(void) {
    return sys_getppid();
}

uid_t getuid(void) {
    return sys_getuid();
}

uid_t geteuid(void) {
    return sys_getuid();
}

int setuid(uid_t uid) {
    int ret = sys_setuid(uid);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}

gid_t getgid(void) {
    return sys_getgid();
}

int setgid(gid_t gid) {
    int ret = sys_setgid(gid);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}

pid_t fork(void) {
    int64_t ret = syscall0(SYS_FORK);
    if (ret < 0) {
        errno = (int)-ret;
        return -1;
    }
    return (pid_t)ret;
}

int execve(const char *pathname, char *const argv[], char *const envp[]) {
    int64_t ret = syscall3(SYS_EXEC, (uint64_t)(uintptr_t)pathname, (uint64_t)(uintptr_t)argv,
                           (uint64_t)(uintptr_t)envp);
    if (ret < 0) {
        errno = (int)-ret;
        return -1;
    }
    return 0;
}

pid_t waitpid(pid_t pid, int *wstatus, int options) {
    int64_t ret =
        syscall3(SYS_WAITPID, (uint64_t)pid, (uint64_t)(uintptr_t)wstatus, (uint64_t)options);
    if (ret < 0) {
        errno = (int)-ret;
        return -1;
    }
    return (pid_t)ret;
}

pid_t wait(int *wstatus) {
    return waitpid(-1, wstatus, 0);
}
