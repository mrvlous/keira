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
#include <time.h>

void sys_sleep(uint32_t ms) {
    syscall1(SYS_SLEEP, (uint64_t)ms);
}

time_t sys_uptime(void) {
    return (time_t)syscall0(SYS_UPTIME);
}
