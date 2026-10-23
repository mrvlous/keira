/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 Execution Delay Utility (sleep)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static void print_usage(void) {
    puts("Usage: sleep NUMBER[s]");
    puts("");
    puts("Description:");
    puts("  Pause execution for NUMBER seconds.");
    puts("");
    puts("Options:");
    puts("  -h, --help     Display this help reference and exit");
}

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "sleep: missing operand\n");
        return 1;
    }

    if (strcmp(argv[1], "-h") == 0 || strcmp(argv[1], "--help") == 0) {
        print_usage();
        return 0;
    }

    char *endptr = NULL;
    long seconds = strtol(argv[1], &endptr, 10);

    if (endptr && *endptr == 's') {
        endptr++;
    }

    if (!endptr || *endptr != '\0' || seconds < 0) {
        fprintf(stderr, "sleep: invalid time interval '%s'\n", argv[1]);
        return 1;
    }

    if (seconds > 0) {
        sleep((unsigned int)seconds);
    }

    return 0;
}
