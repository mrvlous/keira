/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding System Hostname Utility (hostname)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <fcntl.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#define HOSTNAME_PATH "/etc/hostname"
#define HOSTNAME_BUF_SIZE 128

static void print_usage(void) {
    puts("Usage: hostname [NAME]");
    puts("");
    puts("Description:");
    puts("  Display or configure the system hostname.");
    puts("");
    puts("Options:");
    puts("  -h, --help  Display this help reference and exit");
}

int main(int argc, char **argv) {
    if (argc > 1) {
        if (strcmp(argv[1], "-h") == 0 || strcmp(argv[1], "--help") == 0) {
            print_usage();
            return 0;
        }

        const char *new_name = argv[1];
        int fd = open(HOSTNAME_PATH, O_WRONLY | O_CREAT | O_TRUNC, 0644);
        if (fd < 0) {
            fprintf(stderr, "hostname: cannot open '%s' for writing\n", HOSTNAME_PATH);
            return 1;
        }

        size_t len = strlen(new_name);
        write(fd, new_name, len);
        write(fd, "\n", 1);
        close(fd);
        return 0;
    }

    int fd = open(HOSTNAME_PATH, O_RDONLY, 0);
    if (fd < 0) {
        puts("keira");
        return 0;
    }

    char buf[HOSTNAME_BUF_SIZE];
    ssize_t n = read(fd, buf, sizeof(buf) - 1);
    close(fd);

    if (n <= 0) {
        puts("keira");
        return 0;
    }

    buf[n] = '\0';
    char *newline = strchr(buf, '\n');
    if (newline) {
        *newline = '\0';
    }

    puts(buf);
    return 0;
}
