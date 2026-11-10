/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Machine Reboot Utility (reboot)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <errno.h>
#include <stdio.h>
#include <string.h>
#include <sys/reboot.h>
#include <unistd.h>

static void print_usage(void) {
    puts("Usage: reboot [OPTIONS]");
    puts("");
    puts("Description:");
    puts("  Restart the computer via ACPI or processor reset.");
    puts("");
    puts("Options:");
    puts("  -f, --force  Reboot immediately without syncing disks");
    puts("  -h, --help   Display this help reference and exit");
}

int main(int argc, char **argv) {
    int force = 0;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        } else if (strcmp(argv[i], "-f") == 0 || strcmp(argv[i], "--force") == 0) {
            force = 1;
        } else {
            fprintf(stderr, "reboot: unrecognized option '%s'\n", argv[i]);
            fprintf(stderr, "Try 'reboot --help' for more information.\n");
            return 1;
        }
    }

    if (!force) {
        sync();
    }

    if (reboot(RB_AUTOBOOT) < 0) {
        if (errno == EPERM) {
            fprintf(stderr, "reboot: Operation not permitted (must be root)\n");
        } else {
            fprintf(stderr, "reboot: failed to restart system (error %d)\n", errno);
        }
        return 1;
    }

    return 0;
}
