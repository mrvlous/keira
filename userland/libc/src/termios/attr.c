/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
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
#include <sys/syscall.h>
#include <termios.h>

int tcgetattr(int fd, struct termios *termios_p) {
    if (!termios_p) {
        errno = EFAULT;
        return -1;
    }
    int ret = sys_ioctl(fd, TCGETS, termios_p);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}

int tcsetattr(int fd, int optional_actions, const struct termios *termios_p) {
    if (!termios_p) {
        errno = EFAULT;
        return -1;
    }
    unsigned long req = TCSETS;
    if (optional_actions == TCSADRAIN) {
        req = TCSETSW;
    } else if (optional_actions == TCSAFLUSH) {
        req = TCSETSF;
    }
    int ret = sys_ioctl(fd, req, (void *)termios_p);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}
