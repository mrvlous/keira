/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 Text Pattern Matching Utility (grep)
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

#define GREP_BUF_SIZE 4096
#define GREP_LINE_MAX 2048

static int opt_ignore_case = 0;
static int opt_invert_match = 0;
static int opt_line_number = 0;
static int opt_count_only = 0;
static int opt_files_with_matches = 0;

static void print_usage(void) {
    puts("Usage: grep [OPTIONS] PATTERN [FILE...]");
    puts("");
    puts("Description:");
    puts("  Search for PATTERN in each FILE or standard input.");
    puts("");
    puts("Options:");
    puts("  -i             Ignore case distinctions in patterns and input data");
    puts("  -v             Invert the sense of matching, to select non-matching lines");
    puts("  -n             Prefix each line of output with the 1-based line number");
    puts("  -c             Suppress normal output; print a count of matching lines");
    puts("  -l             Suppress normal output; print the name of each matching file");
    puts("  -h, --help     Display this help reference and exit");
}

static const char *str_casestr_custom(const char *haystack, const char *needle) {
    if (!*needle) {
        return haystack;
    }
    for (; *haystack; haystack++) {
        const char *h = haystack;
        const char *n = needle;
        while (*h && *n && tolower((unsigned char)*h) == tolower((unsigned char)*n)) {
            h++;
            n++;
        }
        if (!*n) {
            return haystack;
        }
    }
    return NULL;
}

static int line_matches(const char *line, const char *pattern) {
    int matched;
    if (opt_ignore_case) {
        matched = (str_casestr_custom(line, pattern) != NULL);
    } else {
        matched = (strstr(line, pattern) != NULL);
    }
    return opt_invert_match ? !matched : matched;
}

static int grep_fd(int fd, const char *display_name, const char *pattern, int print_file_prefix) {
    char buf[GREP_BUF_SIZE];
    char line[GREP_LINE_MAX];
    size_t line_len = 0;
    long line_number = 0;
    long match_count = 0;
    ssize_t bytes_read;

    while ((bytes_read = read(fd, buf, sizeof(buf))) > 0) {
        for (ssize_t i = 0; i < bytes_read; i++) {
            char c = buf[i];
            if (c == '\n') {
                line[line_len] = '\0';
                line_number++;

                if (line_matches(line, pattern)) {
                    match_count++;
                    if (opt_files_with_matches) {
                        printf("%s\n", display_name);
                        return 1;
                    }
                    if (!opt_count_only) {
                        if (print_file_prefix) {
                            printf("%s:", display_name);
                        }
                        if (opt_line_number) {
                            printf("%ld:", line_number);
                        }
                        printf("%s\n", line);
                    }
                }
                line_len = 0;
            } else if (line_len + 1 < sizeof(line)) {
                line[line_len++] = c;
            }
        }
    }

    if (line_len > 0) {
        line[line_len] = '\0';
        line_number++;
        if (line_matches(line, pattern)) {
            match_count++;
            if (opt_files_with_matches) {
                printf("%s\n", display_name);
                return 1;
            }
            if (!opt_count_only) {
                if (print_file_prefix) {
                    printf("%s:", display_name);
                }
                if (opt_line_number) {
                    printf("%ld:", line_number);
                }
                printf("%s\n", line);
            }
        }
    }

    if (opt_count_only) {
        if (print_file_prefix) {
            printf("%s:", display_name);
        }
        printf("%ld\n", match_count);
    }

    return (match_count > 0) ? 1 : 0;
}

int main(int argc, char **argv) {
    const char *pattern = NULL;
    int file_start = 0;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        }
        if (argv[i][0] == '-' && argv[i][1] != '\0' && !pattern) {
            const char *opt = argv[i] + 1;
            while (*opt) {
                if (*opt == 'i') {
                    opt_ignore_case = 1;
                } else if (*opt == 'v') {
                    opt_invert_match = 1;
                } else if (*opt == 'n') {
                    opt_line_number = 1;
                } else if (*opt == 'c') {
                    opt_count_only = 1;
                } else if (*opt == 'l') {
                    opt_files_with_matches = 1;
                } else {
                    fprintf(stderr, "grep: invalid option -- '%c'\n", *opt);
                    return 2;
                }
                opt++;
            }
        } else if (!pattern) {
            pattern = argv[i];
            file_start = i + 1;
            break;
        }
    }

    if (!pattern) {
        fprintf(stderr, "grep: missing search pattern\n");
        return 2;
    }

    int files_count = argc - file_start;
    int print_prefix = (files_count > 1);
    int total_matched = 0;
    int exit_status = 1;

    if (files_count == 0) {
        int m = grep_fd(STDIN_FILENO, "(standard input)", pattern, 0);
        if (m > 0) {
            exit_status = 0;
        }
    } else {
        for (int i = file_start; i < argc; i++) {
            if (strcmp(argv[i], "-") == 0) {
                int m = grep_fd(STDIN_FILENO, "(standard input)", pattern, print_prefix);
                if (m > 0) {
                    total_matched += m;
                    exit_status = 0;
                }
                continue;
            }

            int fd = open(argv[i], O_RDONLY, 0);
            if (fd < 0) {
                fprintf(stderr, "grep: %s: No such file or directory\n", argv[i]);
                continue;
            }

            int m = grep_fd(fd, argv[i], pattern, print_prefix);
            if (m > 0) {
                total_matched += m;
                exit_status = 0;
            }
            close(fd);
        }
    }

    return exit_status;
}
