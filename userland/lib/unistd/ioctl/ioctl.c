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
#include <stdarg.h>
#include <stdint.h>
#include <sys/ioctl.h>
#include <sys/syscall.h>

int ioctl(int fd, unsigned long request, ...) {
    va_list ap;
    va_start(ap, request);
    // NOLINTNEXTLINE(clang-analyzer-optin.core.UnconditionalVAArg)
    void *argp = va_arg(ap, void *);
    va_end(ap);

    int ret = (int)syscall3(SYS_IOCTL, (uint64_t)fd, (uint64_t)request, (uint64_t)(uintptr_t)argp);
    if (ret < 0) {
        errno = -ret;
        return -1;
    }
    return ret;
}
