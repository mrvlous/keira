/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdio.h>
#include <string.h>
#include <unistd.h>

void main(void) {
    printf("Keira Text Search Pattern Matcher (grep)\n");

    const char *lines[4] = {"Keira Kernel", "Pure Rust bare-metal hyper-architecture",
                            "Freestanding C Userland Toolchain (KCC)",
                            "Standard POSIX C Library & Microbenchmarks"};

    const char *pattern = "Rust";
    printf("Pattern Query: \"%s\"\n\n", pattern);

    int i = 0;
    int matches = 0;
    while (i < 4) {
        if (strstr(lines[i], pattern)) {
            printf("[%d] %s\n", i + 1, lines[i]);
            matches++;
        }
        i++;
    }

    printf("\nFound %d matching lines.\n", matches);
}
