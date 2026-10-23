/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 Word, Line and Byte Count Utility (wc)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <ctype.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define WC_BUF_SIZE 4096

static int opt_lines = 0;
static int opt_words = 0;
static int opt_bytes = 0;

static void print_usage(void) {
    puts("Usage: wc [OPTIONS] [FILE...]");
    puts("");
    puts("Description:");
    puts("  Print newline, word, and byte counts for each FILE, and a total line if");
    puts(
        "  more than one FILE is specified. With no FILE, or when FILE is -, read standard input.");
    puts("");
    puts("Options:");
    puts("  -l             Print the newline counts");
    puts("  -w             Print the word counts");
    puts("  -c             Print the byte counts");
    puts("  -h, --help     Display this help reference and exit");
}

static void print_counts(long lines, long words, long bytes, const char *display_name) {
    int any_opt = opt_lines || opt_words || opt_bytes;

    if (!any_opt) {
        printf(" %7ld %7ld %7ld", lines, words, bytes);
    } else {
        if (opt_lines) {
            printf(" %7ld", lines);
        }
        if (opt_words) {
            printf(" %7ld", words);
        }
        if (opt_bytes) {
            printf(" %7ld", bytes);
        }
    }

    if (display_name && display_name[0] != '\0') {
        printf(" %s", display_name);
    }
    putchar('\n');
}

static int wc_fd(int fd, const char *display_name, long *out_lines, long *out_words,
                 long *out_bytes) {
    char buf[WC_BUF_SIZE];
    long lines = 0;
    long words = 0;
    long bytes = 0;
    int in_word = 0;
    ssize_t n;

    while ((n = read(fd, buf, sizeof(buf))) > 0) {
        bytes += n;
        for (ssize_t i = 0; i < n; i++) {
            char c = buf[i];
            if (c == '\n') {
                lines++;
            }
            if (isspace((unsigned char)c)) {
                in_word = 0;
            } else if (!in_word) {
                in_word = 1;
                words++;
            }
        }
    }

    if (n < 0) {
        fprintf(stderr, "wc: read error\n");
        return 1;
    }

    *out_lines = lines;
    *out_words = words;
    *out_bytes = bytes;

    print_counts(lines, words, bytes, display_name);
    return 0;
}

int main(int argc, char **argv) {
    int file_start = 0;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        }
        if (argv[i][0] == '-' && argv[i][1] != '\0' && strcmp(argv[i], "-") != 0 && !file_start) {
            const char *opt = argv[i] + 1;
            while (*opt) {
                if (*opt == 'l') {
                    opt_lines = 1;
                } else if (*opt == 'w') {
                    opt_words = 1;
                } else if (*opt == 'c') {
                    opt_bytes = 1;
                } else {
                    fprintf(stderr, "wc: invalid option -- '%c'\n", *opt);
                    return 1;
                }
                opt++;
            }
        } else {
            file_start = i;
            break;
        }
    }

    long total_lines = 0;
    long total_words = 0;
    long total_bytes = 0;
    int files_count = file_start ? (argc - file_start) : 0;
    int exit_status = 0;

    if (files_count == 0) {
        long l = 0, w = 0, b = 0;
        if (wc_fd(STDIN_FILENO, NULL, &l, &w, &b) != 0) {
            exit_status = 1;
        }
    } else {
        for (int i = file_start; i < argc; i++) {
            long l = 0, w = 0, b = 0;
            if (strcmp(argv[i], "-") == 0) {
                if (wc_fd(STDIN_FILENO, "-", &l, &w, &b) == 0) {
                    total_lines += l;
                    total_words += w;
                    total_bytes += b;
                } else {
                    exit_status = 1;
                }
                continue;
            }

            int fd = open(argv[i], O_RDONLY, 0);
            if (fd < 0) {
                fprintf(stderr, "wc: %s: No such file or directory\n", argv[i]);
                exit_status = 1;
                continue;
            }

            if (wc_fd(fd, argv[i], &l, &w, &b) == 0) {
                total_lines += l;
                total_words += w;
                total_bytes += b;
            } else {
                exit_status = 1;
            }
            close(fd);
        }

        if (files_count > 1) {
            print_counts(total_lines, total_words, total_bytes, "total");
        }
    }

    return exit_status;
}
