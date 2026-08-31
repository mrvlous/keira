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
#include <signal.h>
#include <syscall.h>

static sighandler_t sig_table[32] = {0};

sighandler_t signal(int signum, sighandler_t handler) {
    if (signum < 1 || signum >= 32) {
        errno = EINVAL;
        return SIG_ERR;
    }
    sighandler_t old = sig_table[signum];
    sig_table[signum] = handler;
    return old;
}

int raise(int sig) {
    if (sig < 1 || sig >= 32) {
        errno = EINVAL;
        return -1;
    }
    if (sig_table[sig] && sig_table[sig] != SIG_DFL && sig_table[sig] != SIG_IGN) {
        sig_table[sig](sig);
    }
    return 0;
}

int kill(pid_t pid, int sig) {
    (void)pid;
    (void)sig;
    return 0;
}
