/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 Unmount Utility (umount)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mount.h>
#include <unistd.h>

static void print_usage(void) {
    puts("Usage: umount [OPTION]... DIRECTORY|DEVICE");
    puts("       umount [-h]");
    puts("");
    puts("Description:");
    puts("  Unmount a filesystem from the virtual directory hierarchy.");
    puts("");
    puts("Options:");
    puts("  -f, --force    force unmount");
    puts("  -h, --help     display this help message and exit");
}

int main(int argc, char **argv) {
    const char *target = NULL;
    int force = 0;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        } else if (strcmp(argv[i], "-f") == 0 || strcmp(argv[i], "--force") == 0) {
            force = 1;
        } else if (argv[i][0] == '-') {
            fprintf(stderr, "umount: unrecognized option '%s'\n", argv[i]);
            return 1;
        } else if (!target) {
            target = argv[i];
        }
    }

    if (!target) {
        fprintf(stderr, "umount: missing operand\n");
        fprintf(stderr, "Try 'umount --help' for more information.\n");
        return 1;
    }

    int res = umount2(target, force ? 1 : 0);
    if (res < 0) {
        fprintf(stderr, "umount: unmounting %s failed (error %d)\n", target, res);
        return 1;
    }

    printf("umount: successfully unmounted %s\n", target);
    return 0;
}
