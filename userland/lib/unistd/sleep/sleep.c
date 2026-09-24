/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <syscall.h>
#include <unistd.h>

unsigned int sleep(unsigned int seconds) {
    sys_sleep(seconds * 1000);
    return 0;
}

int usleep(unsigned int usec) {
    sys_sleep(usec / 1000);
    return 0;
}
