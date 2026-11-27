/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel Module Listing Utility (lsmod)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

#include <fcntl.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#define BUF_SIZE 2048

static void print_usage(void) {
    puts("Usage: lsmod [OPTIONS]");
    puts("");
    puts("Description:");
    puts("  Show status of loaded modules in the Linux-grade kernel.");
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

    int fd = open("/proc/modules", O_RDONLY, 0);
    if (fd < 0) {
        fprintf(stderr, "lsmod: failed to open /proc/modules\n");
        return 1;
    }

    char buf[BUF_SIZE];
    ssize_t n = read(fd, buf, sizeof(buf) - 1);
    close(fd);

    if (n < 0) {
        fprintf(stderr, "lsmod: error reading /proc/modules\n");
        return 1;
    }
    buf[n] = '\0';

    printf("%-20s %8s %8s %-10s %s\n", "Module", "Size", "Used by", "State", "Address");

    char *line = buf;
    while (line && *line) {
        char *next = strchr(line, '\n');
        if (next) {
            *next = '\0';
            next++;
        }

        if (*line) {
            char name[64] = "";
            char size[32] = "";
            char ref[32] = "";
            char dash[16] = "";
            char state[32] = "";
            char addr[64] = "";

            int matched =
                sscanf(line, "%63s %31s %31s %15s %31s %63s", name, size, ref, dash, state, addr);
            if (matched >= 5) {
                printf("%-20s %8s %8s %-10s %s\n", name, size, ref, state, addr);
            } else {
                puts(line);
            }
        }

        line = next;
    }

    return 0;
}
