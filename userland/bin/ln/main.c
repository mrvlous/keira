/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 Link Utility (ln)
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
#include <unistd.h>

static void print_usage(void) {
    puts("Usage: ln [OPTION]... TARGET LINK_NAME");
    puts("");
    puts("Description:");
    puts("  Create a link to TARGET with the name LINK_NAME.");
    puts("");
    puts("Options:");
    puts("  -s, --symbolic     make symbolic links instead of hard links");
    puts("  -f, --force        remove existing destination files");
    puts("  -v, --verbose      print name of each linked file");
    puts("  -h, --help         display this help message and exit");
}

int main(int argc, char **argv) {
    int symbolic = 0;
    int force = 0;
    int verbose = 0;
    const char *target = NULL;
    const char *linkname = NULL;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        } else if (strcmp(argv[i], "-s") == 0 || strcmp(argv[i], "--symbolic") == 0) {
            symbolic = 1;
        } else if (strcmp(argv[i], "-f") == 0 || strcmp(argv[i], "--force") == 0) {
            force = 1;
        } else if (strcmp(argv[i], "-v") == 0 || strcmp(argv[i], "--verbose") == 0) {
            verbose = 1;
        } else if (strcmp(argv[i], "-sf") == 0 || strcmp(argv[i], "-fs") == 0) {
            symbolic = 1;
            force = 1;
        } else if (strcmp(argv[i], "-sv") == 0 || strcmp(argv[i], "-vs") == 0) {
            symbolic = 1;
            verbose = 1;
        } else if (strcmp(argv[i], "-sfv") == 0 || strcmp(argv[i], "-svf") == 0) {
            symbolic = 1;
            force = 1;
            verbose = 1;
        } else if (argv[i][0] == '-') {
            fprintf(stderr, "ln: unrecognized option '%s'\n", argv[i]);
            fprintf(stderr, "Try 'ln --help' for more information.\n");
            return 1;
        } else {
            if (!target) {
                target = argv[i];
            } else if (!linkname) {
                linkname = argv[i];
            } else {
                fprintf(stderr, "ln: extra operand '%s'\n", argv[i]);
                fprintf(stderr, "Try 'ln --help' for more information.\n");
                return 1;
            }
        }
    }

    if (!target) {
        fprintf(stderr, "ln: missing file operand\n");
        fprintf(stderr, "Try 'ln --help' for more information.\n");
        return 1;
    }

    if (!linkname) {
        fprintf(stderr, "ln: missing destination file operand after '%s'\n", target);
        fprintf(stderr, "Try 'ln --help' for more information.\n");
        return 1;
    }

    if (force) {
        unlink(linkname);
    }

    if (symlink(target, linkname) != 0) {
        fprintf(stderr, "ln: failed to create %slink '%s': ", symbolic ? "symbolic " : "",
                linkname);
        if (errno == EEXIST) {
            fprintf(stderr, "File exists\n");
        } else if (errno == EACCES) {
            fprintf(stderr, "Permission denied\n");
        } else if (errno == ENOENT) {
            fprintf(stderr, "No such file or directory\n");
        } else if (errno == ENAMETOOLONG) {
            fprintf(stderr, "File name too long\n");
        } else if (errno == ENOSPC) {
            fprintf(stderr, "No space left on device\n");
        } else {
            fprintf(stderr, "Unknown error (%d)\n", errno);
        }
        return 1;
    }

    if (verbose) {
        printf("'%s' -> '%s'\n", linkname, target);
    }

    return 0;
}
