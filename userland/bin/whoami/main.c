/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Canonical Whoami Utility
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
    puts("Usage: whoami [OPTIONS]");
    puts("");
    puts("Description:");
    puts("  Print the effective user identity associated with the current user ID.");
    puts("");
    puts("Options:");
    puts("  -h, --help     Display this help reference and exit");
    puts("  -v, --version  Display version information and exit");
}

int main(int argc, char **argv) {
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        }
        if (strcmp(argv[i], "-v") == 0 || strcmp(argv[i], "--version") == 0) {
            puts("whoami (Keira Coreutils) 0.7.0");
            return 0;
        }
        fprintf(stderr, "whoami: unrecognized option '%s'\n", argv[i]);
        fprintf(stderr, "Try 'whoami --help' for more information.\n");
        return 1;
    }

    uid_t euid = geteuid();
    if (euid == 0) {
        puts("root");
    } else {
        printf("%u\n", (unsigned int)euid);
    }

    return 0;
}
