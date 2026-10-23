/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 Head Stream Truncation Utility (head)
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

#define HEAD_DEFAULT_LINES 10
#define HEAD_BUF_SIZE 4096

static void print_usage(void) {
    puts("Usage: head [OPTIONS] [FILE...]");
    puts("");
    puts("Description:");
    puts("  Print the first 10 lines of each FILE to standard output.");
    puts("  With more than one FILE, precede each with a header giving the file name.");
    puts("  With no FILE, or when FILE is -, read standard input.");
    puts("");
    puts("Options:");
    puts("  -n <lines>     Print the first <lines> lines instead of the first 10");
    puts("  -h, --help     Display this help reference and exit");
}

static int head_fd(int fd, long max_lines) {
    char buf[HEAD_BUF_SIZE];
    long lines_printed = 0;
    ssize_t bytes_read;

    while (lines_printed < max_lines && (bytes_read = read(fd, buf, sizeof(buf))) > 0) {
        ssize_t chunk_start = 0;
        for (ssize_t i = 0; i < bytes_read; i++) {
            if (buf[i] == '\n') {
                lines_printed++;
                if (lines_printed >= max_lines) {
                    write(STDOUT_FILENO, buf + chunk_start, (size_t)(i - chunk_start + 1));
                    return 0;
                }
            }
        }
        write(STDOUT_FILENO, buf + chunk_start, (size_t)(bytes_read - chunk_start));
    }

    return 0;
}

int main(int argc, char **argv) {
    long max_lines = HEAD_DEFAULT_LINES;
    int file_start = 0;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        }
        if (strcmp(argv[i], "-n") == 0) {
            if (i + 1 < argc) {
                max_lines = atol(argv[++i]);
                if (max_lines <= 0) {
                    fprintf(stderr, "head: invalid number of lines: '%s'\n", argv[i]);
                    return 1;
                }
            } else {
                fprintf(stderr, "head: option requires an argument -- 'n'\n");
                return 1;
            }
        } else if (argv[i][0] == '-' && argv[i][1] >= '0' && argv[i][1] <= '9') {
            max_lines = atol(argv[i] + 1);
            if (max_lines <= 0) {
                fprintf(stderr, "head: invalid number of lines: '%s'\n", argv[i] + 1);
                return 1;
            }
        } else {
            file_start = i;
            break;
        }
    }

    int files_count = file_start ? (argc - file_start) : 0;
    int exit_status = 0;

    if (files_count == 0) {
        head_fd(STDIN_FILENO, max_lines);
    } else {
        int first = 1;
        for (int i = file_start; i < argc; i++) {
            if (files_count > 1) {
                if (!first) {
                    putchar('\n');
                }
                printf("==> %s <==\n", argv[i]);
                first = 0;
            }

            if (strcmp(argv[i], "-") == 0) {
                head_fd(STDIN_FILENO, max_lines);
                continue;
            }

            int fd = open(argv[i], O_RDONLY, 0);
            if (fd < 0) {
                fprintf(stderr, "head: %s: No such file or directory\n", argv[i]);
                exit_status = 1;
                continue;
            }

            head_fd(fd, max_lines);
            close(fd);
        }
    }

    return exit_status;
}
