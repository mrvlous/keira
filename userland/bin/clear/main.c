/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Terminal Clear Utility (clear)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdio.h>
#include <string.h>
#include <unistd.h>

static void print_usage(void) {
    puts("Usage: clear [OPTIONS]");
    puts("");
    puts("Description:");
    puts("  Clear the terminal screen buffer.");
    puts("");
    puts("Options:");
    puts("  -h, --help  Display this help reference and exit");
}

int main(int argc, char **argv) {
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        }
    }

    /* ANSI escape sequence: Clear entire screen (2J) and cursor home (H) */
    const char *ansi_clear = "\033[2J\033[H";
    write(STDOUT_FILENO, ansi_clear, strlen(ansi_clear));
    return 0;
}
