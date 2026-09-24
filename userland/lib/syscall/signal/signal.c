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

int sys_kill(pid_t pid, int sig) {
    return (int)syscall2(SYS_KILL, (uint64_t)pid, (uint64_t)sig);
}

int sys_sigprocmask(int how, const void *set, void *oldset) {
    return (int)syscall3(SYS_SIGPROCMASK, (uint64_t)how, (uint64_t)(uintptr_t)set,
                         (uint64_t)(uintptr_t)oldset);
}

int sys_sigpending(void *set) {
    return (int)syscall1(SYS_SIGPENDING, (uint64_t)(uintptr_t)set);
}
