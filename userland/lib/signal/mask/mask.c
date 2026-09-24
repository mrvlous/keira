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
#include <signal.h>
#include <syscall.h>

int sigprocmask(int how, const sigset_t *set, sigset_t *oldset) {
    if (how != SIG_BLOCK && how != SIG_UNBLOCK && how != SIG_SETMASK) {
        errno = EINVAL;
        return -1;
    }
    int ret = sys_sigprocmask(how, set, oldset);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}

int sigpending(sigset_t *set) {
    if (!set) {
        errno = EFAULT;
        return -1;
    }
    int ret = sys_sigpending(set);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}

int sigemptyset(sigset_t *set) {
    if (!set) {
        errno = EINVAL;
        return -1;
    }
    *set = 0;
    return 0;
}

int sigfillset(sigset_t *set) {
    if (!set) {
        errno = EINVAL;
        return -1;
    }
    *set = 0xFFFFFFFF;
    return 0;
}

int sigaddset(sigset_t *set, int signum) {
    if (!set || signum < 1 || signum >= 32) {
        errno = EINVAL;
        return -1;
    }
    *set |= (1U << signum);
    return 0;
}

int sigdelset(sigset_t *set, int signum) {
    if (!set || signum < 1 || signum >= 32) {
        errno = EINVAL;
        return -1;
    }
    *set &= ~(1U << signum);
    return 0;
}

int sigismember(const sigset_t *set, int signum) {
    if (!set || signum < 1 || signum >= 32) {
        errno = EINVAL;
        return -1;
    }
    return (*set & (1U << signum)) ? 1 : 0;
}
