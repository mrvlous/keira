/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include "stdlib.h"

#include "string.h"
#include "syscall.h"

char *getenv(const char *name) {
    static char env_val_buf[256];
    int len = sys_getenv(name, env_val_buf, sizeof(env_val_buf));
    if (len < 0) {
        return NULL;
    }
    return env_val_buf;
}

int setenv(const char *name, const char *value, int overwrite) {
    (void)overwrite;
    return sys_setenv(name, value);
}

int http_get(const char *url, void *buf, int max_len) {
    return sys_http_get(url, buf, max_len);
}
