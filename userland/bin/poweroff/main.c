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

/*
 * Keira Kernel - Freestanding Machine Poweroff Utility (poweroff)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 */

#include <errno.h>
#include <stdio.h>
#include <string.h>
#include <sys/reboot.h>
#include <unistd.h>

static void print_usage(void) {
    puts("Usage: poweroff [OPTIONS]");
    puts("");
    puts("Description:");
    puts("  Power down the machine via ACPI S5 sleep state.");
    puts("");
    puts("Options:");
    puts("  -f, --force  Power off immediately without syncing disks");
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
            fprintf(stderr, "poweroff: unrecognized option '%s'\n", argv[i]);
            fprintf(stderr, "Try 'poweroff --help' for more information.\n");
            return 1;
        }
    }

    if (!force) {
        sync();
    }

    if (reboot(RB_POWER_OFF) < 0) {
        if (errno == EPERM) {
            fprintf(stderr, "poweroff: Operation not permitted (must be root)\n");
        } else {
            fprintf(stderr, "poweroff: failed to power off system (error %d)\n", errno);
        }
        return 1;
    }

    return 0;
}
