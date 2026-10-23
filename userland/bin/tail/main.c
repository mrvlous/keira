/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 Tail Stream Truncation Utility (tail)
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

#define TAIL_DEFAULT_LINES 10
#define TAIL_BUF_SIZE 4096
#define TAIL_LINE_MAX 2048
#define TAIL_RING_MAX 1024

static void print_usage(void) {
    puts("Usage: tail [OPTIONS] [FILE...]");
    puts("");
    puts("Description:");
    puts("  Print the last 10 lines of each FILE to standard output.");
    puts("  With more than one FILE, precede each with a header giving the file name.");
    puts("  With no FILE, or when FILE is -, read standard input.");
    puts("");
    puts("Options:");
    puts("  -n <lines>     Output the last <lines> lines, instead of the last 10");
    puts("  -h, --help     Display this help reference and exit");
}

static int tail_fd(int fd, long max_lines) {
    if (max_lines <= 0) {
        return 0;
    }
    if (max_lines > TAIL_RING_MAX) {
        max_lines = TAIL_RING_MAX;
    }

    char *ring[TAIL_RING_MAX];
    for (long i = 0; i < max_lines; i++) {
        ring[i] = NULL;
    }

    long ring_idx = 0;
    long total_lines = 0;

    char buf[TAIL_BUF_SIZE];
    char current_line[TAIL_LINE_MAX];
    size_t line_len = 0;
    ssize_t n;

    while ((n = read(fd, buf, sizeof(buf))) > 0) {
        for (ssize_t i = 0; i < n; i++) {
            char c = buf[i];
            if (c == '\n') {
                current_line[line_len] = '\0';

                if (ring[ring_idx]) {
                    free(ring[ring_idx]);
                }
                ring[ring_idx] = strdup(current_line);
                ring_idx = (ring_idx + 1) % max_lines;
                total_lines++;
                line_len = 0;
            } else if (line_len + 1 < sizeof(current_line)) {
                current_line[line_len++] = c;
            }
        }
    }

    if (line_len > 0) {
        current_line[line_len] = '\0';
        if (ring[ring_idx]) {
            free(ring[ring_idx]);
        }
        ring[ring_idx] = strdup(current_line);
        ring_idx = (ring_idx + 1) % max_lines;
        total_lines++;
    }

    long count_to_print = (total_lines < max_lines) ? total_lines : max_lines;
    long start_idx = (total_lines < max_lines) ? 0 : ring_idx;

    for (long i = 0; i < count_to_print; i++) {
        long idx = (start_idx + i) % max_lines;
        if (ring[idx]) {
            puts(ring[idx]);
            free(ring[idx]);
            ring[idx] = NULL;
        }
    }

    for (long i = 0; i < max_lines; i++) {
        if (ring[i]) {
            free(ring[i]);
            ring[i] = NULL;
        }
    }

    return 0;
}

int main(int argc, char **argv) {
    long max_lines = TAIL_DEFAULT_LINES;
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
                    fprintf(stderr, "tail: invalid number of lines: '%s'\n", argv[i]);
                    return 1;
                }
            } else {
                fprintf(stderr, "tail: option requires an argument -- 'n'\n");
                return 1;
            }
        } else if (argv[i][0] == '-' && argv[i][1] >= '0' && argv[i][1] <= '9') {
            max_lines = atol(argv[i] + 1);
            if (max_lines <= 0) {
                fprintf(stderr, "tail: invalid number of lines: '%s'\n", argv[i] + 1);
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
        tail_fd(STDIN_FILENO, max_lines);
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
                tail_fd(STDIN_FILENO, max_lines);
                continue;
            }

            int fd = open(argv[i], O_RDONLY, 0);
            if (fd < 0) {
                fprintf(stderr, "tail: %s: No such file or directory\n", argv[i]);
                exit_status = 1;
                continue;
            }

            tail_fd(fd, max_lines);
            close(fd);
        }
    }

    return exit_status;
}
