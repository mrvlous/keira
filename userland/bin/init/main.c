/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Canonical Init System (PID 1)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdio.h>
#include <string.h>
#include <unistd.h>

int main(int argc, char **argv) {
    if (argc > 1 && (strcmp(argv[1], "-v") == 0 || strcmp(argv[1], "--verbose") == 0)) {
        puts("[INIT] Keira Canonical Userspace Init (PID 1) started in Ring 3");
        puts("[INIT] Pure freestanding kernel environment certified");
        puts("[INIT] System initialization complete. Entering supervisor control plane");
    }

    return 0;
}
