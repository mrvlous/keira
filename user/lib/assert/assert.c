/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>

void __assert_fail(const char *assertion, const char *file, unsigned int line,
                   const char *function) {
    const char *fn = function;
    if (!fn) {
        fn = "unknown";
    }
    printf("[ASSERTION FAILED] %s in %s (%s:%u)\n", assertion, fn, file, line);
    abort();
}
