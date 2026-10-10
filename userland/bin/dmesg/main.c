/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel Ring Buffer Display Utility (dmesg)
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

#define DMESG_BUF_SIZE 1024

static void print_usage(void) {
    puts("Usage: dmesg [OPTIONS]");
    puts("");
    puts("Description:");
    puts("  Display kernel diagnostic and boot ring buffer messages.");
    puts("");
    puts("Options:");
    puts("  -h, --help  Display this help reference and exit");
}

int main(int argc, char **argv) {
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        }
    }

    const char *log_files[] = {"/var/log/system.log", "/var/log/boot.log"};
    int printed = 0;

    for (size_t f = 0; f < sizeof(log_files) / sizeof(log_files[0]); f++) {
        int fd = open(log_files[f], O_RDONLY, 0);
        if (fd >= 0) {
            char buf[DMESG_BUF_SIZE];
            ssize_t n;
            while ((n = read(fd, buf, sizeof(buf))) > 0) {
                write(STDOUT_FILENO, buf, (size_t)n);
                printed = 1;
            }
            close(fd);
        }
    }

    if (!printed) {
        puts("[dmesg] No active kernel log records available.");
    }

    return 0;
}
