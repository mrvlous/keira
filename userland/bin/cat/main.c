/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 File Concatenation Utility (cat)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define CAT_BUF_SIZE 1024

static int cat_fd(int fd, const char *name) {
    char buf[CAT_BUF_SIZE];
    while (1) {
        ssize_t n = read(fd, buf, sizeof(buf));
        if (n < 0) {
            printf("cat: %s: Read error\n", name);
            return 1;
        }
        if (n == 0) {
            break;
        }

        ssize_t written = 0;
        while (written < n) {
            ssize_t w = write(STDOUT_FILENO, buf + written, (size_t)(n - written));
            if (w <= 0) {
                printf("cat: Write error on stdout\n");
                return 1;
            }
            written += w;
        }
    }
    return 0;
}

int main(int argc, char **argv) {
    if (argc >= 2 && (strcmp(argv[1], "-h") == 0 || strcmp(argv[1], "--help") == 0)) {
        puts("Usage: cat [FILE...]");
        puts("");
        puts("Description:");
        puts("  Concatenate FILE(s) to standard output.");
        puts("  With no FILE, or when FILE is -, read standard input.");
        puts("");
        puts("Options:");
        puts("  -h, --help     Display this help message and exit");
        return 0;
    }

    if (argc == 1) {
        return cat_fd(STDIN_FILENO, "standard input");
    }

    int exit_status = 0;
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-") == 0) {
            if (cat_fd(STDIN_FILENO, "standard input") != 0) {
                exit_status = 1;
            }
            continue;
        }

        int fd = open(argv[i], O_RDONLY, 0);
        if (fd < 0) {
            printf("cat: %s: No such file or directory\n", argv[i]);
            exit_status = 1;
            continue;
        }

        if (cat_fd(fd, argv[i]) != 0) {
            exit_status = 1;
        }
        close(fd);
    }

    return exit_status;
}
