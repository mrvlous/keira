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

void sys_exit(int status) {
    syscall1(SYS_EXIT, (uint64_t)(uint32_t)status);
    while (1) {
        __asm__ volatile("pause");
    }
}

pid_t sys_getpid(void) {
    return (pid_t)syscall0(SYS_GETPID);
}

pid_t sys_getppid(void) {
    return (pid_t)syscall0(SYS_GETPPID);
}

uid_t sys_getuid(void) {
    return (uid_t)syscall0(SYS_GETUID);
}

int sys_setuid(uid_t uid) {
    return (int)syscall1(SYS_SETUID, (uint64_t)uid);
}

gid_t sys_getgid(void) {
    return (gid_t)syscall0(SYS_GETGID);
}

int sys_setgid(gid_t gid) {
    return (int)syscall1(SYS_SETGID, (uint64_t)gid);
}

pid_t sys_fork(void) {
    return (pid_t)syscall0(SYS_FORK);
}
