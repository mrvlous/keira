/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 File Timestamp Utility (touch)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static void print_usage(void) {
    puts("Usage: touch [OPTION]... FILE...");
    puts("");
    puts("Description:");
    puts("  Update the access and modification times of each FILE to the current time.");
    puts("  A FILE argument that does not exist is created empty.");
    puts("");
    puts("Options:");
    puts("  -c, --no-create  do not create any files");
    puts("  -h, --help       display this help message and exit");
}

static int touch_file(const char *path, int no_create) {
    int flags = O_WRONLY;
    if (!no_create) {
        flags |= O_CREAT;
    }
    int fd = open(path, flags, 0644);
    if (fd < 0) {
        if (no_create && errno == ENOENT) {
            return 0;
        }
        printf("touch: cannot touch '%s': ", path);
        if (errno == ENOENT) {
            puts("No such file or directory");
        } else if (errno == EACCES) {
            puts("Permission denied");
        } else {
            puts("Cannot touch file");
        }
        return 1;
    }
    close(fd);
    return 0;
}

int main(int argc, char **argv) {
    int opt_no_create = 0;
    const char *files[32];
    int file_count = 0;

    for (int i = 1; i < argc; i++) {
        char *arg = argv[i];
        if (strcmp(arg, "-h") == 0 || strcmp(arg, "--help") == 0) {
            print_usage();
            return 0;
        } else if (strcmp(arg, "-c") == 0 || strcmp(arg, "--no-create") == 0) {
            opt_no_create = 1;
        } else if (arg[0] == '-' && arg[1] != '\0') {
            printf("touch: unrecognized option '%s'\n", arg);
            puts("Try 'touch --help' for more information.");
            return 1;
        } else {
            if (file_count < 32) {
                files[file_count++] = arg;
            }
        }
    }

    if (file_count == 0) {
        puts("touch: missing file operand");
        puts("Try 'touch --help' for more information.");
        return 1;
    }

    int exit_status = 0;
    for (int i = 0; i < file_count; i++) {
        if (touch_file(files[i], opt_no_create) != 0) {
            exit_status = 1;
        }
    }

    return exit_status;
}
