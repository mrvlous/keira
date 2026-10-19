/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Canonical ID Utility
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static void print_usage(void) {
    puts("Usage: id [OPTIONS]... [USER/UID]");
    puts("");
    puts("Description:");
    puts("  Print real and effective user ID (UID) and group ID (GID) from kernel.");
    puts("");
    puts("Options:");
    puts("  -u, --user     Print only the effective user ID");
    puts("  -g, --group    Print only the effective group ID");
    puts("  -G, --groups   Print all group IDs");
    puts("  -r, --real     Print the real ID instead of the effective ID");
    puts("  -h, --help     Display this help reference and exit");
    puts("  -v, --version  Display version information and exit");
}

int main(int argc, char **argv) {
    bool opt_u = false;
    bool opt_g = false;
    bool opt_G = false;
    bool opt_r = false;
    const char *target = NULL;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        }
        if (strcmp(argv[i], "-v") == 0 || strcmp(argv[i], "--version") == 0) {
            puts("id (Keira Coreutils) 0.7.0");
            return 0;
        }
        if (argv[i][0] == '-') {
            for (size_t j = 1; j < strlen(argv[i]); j++) {
                if (argv[i][j] == 'u') {
                    opt_u = true;
                } else if (argv[i][j] == 'g') {
                    opt_g = true;
                } else if (argv[i][j] == 'G') {
                    opt_G = true;
                } else if (argv[i][j] == 'r') {
                    opt_r = true;
                } else {
                    fprintf(stderr, "id: invalid option -- '%c'\n", argv[i][j]);
                    fprintf(stderr, "Try 'id --help' for more information.\n");
                    return 1;
                }
            }
        } else {
            if (!target) {
                target = argv[i];
            } else {
                fprintf(stderr, "id: extra operand '%s'\n", argv[i]);
                return 1;
            }
        }
    }

    uid_t uid = 0;
    uid_t euid = 0;
    gid_t gid = 0;
    gid_t egid = 0;

    if (target) {
        if (strcmp(target, "root") == 0) {
            uid = 0;
            euid = 0;
            gid = 0;
            egid = 0;
        } else {
            uid = (uid_t)atoi(target);
            euid = uid;
            gid = (gid_t)uid;
            egid = (gid_t)uid;
        }
    } else {
        uid = getuid();
        euid = geteuid();
        gid = getgid();
        egid = getegid();
    }

    if (opt_u) {
        printf("%u\n", (unsigned int)(opt_r ? uid : euid));
        return 0;
    }

    if (opt_g) {
        printf("%u\n", (unsigned int)(opt_r ? gid : egid));
        return 0;
    }

    if (opt_G) {
        printf("%u\n", (unsigned int)(opt_r ? gid : egid));
        return 0;
    }

    /* Standard kernel credential representation */
    if (uid == 0) {
        printf("uid=0(root) gid=0(root)");
    } else {
        printf("uid=%u gid=%u", (unsigned int)uid, (unsigned int)gid);
    }

    if (euid != uid) {
        if (euid == 0) {
            printf(" euid=0(root)");
        } else {
            printf(" euid=%u", (unsigned int)euid);
        }
    }

    if (egid != gid) {
        if (egid == 0) {
            printf(" egid=0(root)");
        } else {
            printf(" egid=%u", (unsigned int)egid);
        }
    }

    if (gid == 0) {
        printf(" groups=0(root)\n");
    } else {
        printf(" groups=%u\n", (unsigned int)gid);
    }

    return 0;
}
