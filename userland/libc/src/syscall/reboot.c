// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
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
#include <stdint.h>
#include <sys/reboot.h>
#include <sys/syscall.h>
#include <unistd.h>

int sys_reboot(int magic1, int magic2, int cmd, void *arg) {
    int64_t ret = syscall4(SYS_REBOOT, (uint64_t)(uint32_t)magic1, (uint64_t)(uint32_t)magic2,
                           (uint64_t)(uint32_t)cmd, (uint64_t)(uintptr_t)arg);
    return (int)ret;
}

int reboot(int cmd) {
    int ret = sys_reboot((int)LINUX_REBOOT_MAGIC1, (int)LINUX_REBOOT_MAGIC2, cmd, NULL);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return 0;
}
