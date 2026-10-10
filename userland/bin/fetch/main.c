/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Ring 3 HTTP Fetch Utility (fetch)
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
#include <sys/syscall.h>
#include <unistd.h>

#define FETCH_BUF_SIZE 4096

static void print_usage(void) {
    puts("Usage: fetch [OPTIONS] <URL>");
    puts("");
    puts("Description:");
    puts("  Freestanding Ring 3 HTTP client utility to retrieve web resources.");
    puts("");
    puts("Options:");
    puts("  -o <file>      Write payload to local file instead of standard output");
    puts("  -I, --head     Show response headers and status line only");
    puts("  -v, --verbose  Display connection and payload transfer telemetry");
    puts("  -h, --help     Display this help reference and exit");
}

int main(int argc, char **argv) {
    const char *url = NULL;
    const char *out_file = NULL;
    int head_only = 0;
    int verbose = 0;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        }
        if (strcmp(argv[i], "-v") == 0 || strcmp(argv[i], "--verbose") == 0) {
            verbose = 1;
        } else if (strcmp(argv[i], "-I") == 0 || strcmp(argv[i], "--head") == 0) {
            head_only = 1;
        } else if (strcmp(argv[i], "-o") == 0) {
            if (i + 1 < argc) {
                out_file = argv[++i];
            } else {
                fprintf(stderr, "fetch: option '-o' requires an output file argument\n");
                return 1;
            }
        } else if (argv[i][0] != '-') {
            url = argv[i];
        } else {
            fprintf(stderr, "fetch: unrecognized option '%s'\n", argv[i]);
            return 1;
        }
    }

    if (!url) {
        print_usage();
        return 1;
    }

    if (verbose) {
        printf("[fetch] Requesting resource: %s\n", url);
    }

    char *buf = (char *)malloc(FETCH_BUF_SIZE);
    if (!buf) {
        fprintf(stderr, "fetch: memory allocation error\n");
        return 1;
    }
    memset(buf, 0, FETCH_BUF_SIZE);

    ssize_t n = sys_http_get(url, buf, FETCH_BUF_SIZE - 1);
    if (n <= 0) {
        fprintf(stderr, "fetch: failed to retrieve '%s': network or host error\n", url);
        free(buf);
        return 1;
    }

    buf[n] = '\0';

    if (verbose) {
        printf("[fetch] Successfully received %ld bytes\n", (long)n);
    }

    if (head_only) {
        char *header_end = strstr(buf, "\r\n\r\n");
        if (header_end) {
            size_t head_len = (size_t)(header_end - buf) + 2;
            write(STDOUT_FILENO, buf, head_len);
            putchar('\n');
        } else {
            char *line_end = strchr(buf, '\n');
            if (line_end) {
                size_t line_len = (size_t)(line_end - buf) + 1;
                write(STDOUT_FILENO, buf, line_len);
            } else {
                puts(buf);
            }
        }
        free(buf);
        return 0;
    }

    if (out_file) {
        int fd = open(out_file, O_WRONLY | O_CREAT | O_TRUNC, 0644);
        if (fd < 0) {
            fprintf(stderr, "fetch: cannot open output file '%s' for writing\n", out_file);
            free(buf);
            return 1;
        }

        /* If response contains HTTP headers, extract body */
        char *body_start = strstr(buf, "\r\n\r\n");
        const char *data_to_write = buf;
        size_t bytes_to_write = (size_t)n;

        if (body_start) {
            body_start += 4;
            data_to_write = body_start;
            bytes_to_write = (size_t)(n - (body_start - buf));
        }

        ssize_t written = write(fd, data_to_write, bytes_to_write);
        close(fd);

        if (written < 0 || (size_t)written != bytes_to_write) {
            fprintf(stderr, "fetch: write failure to file '%s'\n", out_file);
            free(buf);
            return 1;
        }

        if (verbose) {
            printf("[fetch] Wrote %ld payload bytes to '%s'\n", (long)written, out_file);
        }
    } else {
        write(STDOUT_FILENO, buf, (size_t)n);
    }

    free(buf);
    return 0;
}
