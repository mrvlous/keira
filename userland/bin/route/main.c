/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 IP Routing Table Utility (route)
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

static void print_usage(void) {
    puts("Usage: route [OPTIONS]");
    puts("");
    puts("Description:");
    puts("  Display or manipulate the kernel IPv4 routing table.");
    puts("");
    puts("Options:");
    puts("  -n, --numeric  do not resolve hostnames (numeric display)");
    puts("  -h, --help     display this help message and exit");
}

static void format_hex_ip(unsigned int hex, char *out, size_t size) {
    unsigned char b0 = (unsigned char)(hex & 0xFF);
    unsigned char b1 = (unsigned char)((hex >> 8) & 0xFF);
    unsigned char b2 = (unsigned char)((hex >> 16) & 0xFF);
    unsigned char b3 = (unsigned char)((hex >> 24) & 0xFF);
    snprintf(out, size, "%u.%u.%u.%u", b0, b1, b2, b3);
}

int main(int argc, char **argv) {
    for (int i = 1; i < argc; i++) {
        char *arg = argv[i];
        if (strcmp(arg, "-h") == 0 || strcmp(arg, "--help") == 0) {
            print_usage();
            return 0;
        }
        if (strcmp(arg, "-n") == 0 || strcmp(arg, "--numeric") == 0) {
            continue;
        }
    }

    int fd = open("/proc/net/route", O_RDONLY, 0);
    if (fd < 0) {
        fprintf(stderr, "route: cannot open /proc/net/route: No such file or directory\n");
        return 1;
    }

    char buf[1024];
    ssize_t n = read(fd, buf, sizeof(buf) - 1);
    close(fd);

    if (n <= 0) {
        fprintf(stderr, "route: /proc/net/route empty or read error\n");
        return 1;
    }
    buf[n] = '\0';

    puts("Kernel IP routing table");
    puts("Destination     Gateway         Genmask         Flags Metric Ref    Use Iface");

    char *line = strtok(buf, "\n");
    int line_num = 0;
    while (line) {
        line_num++;
        if (line_num > 1) {
            char iface[32];
            unsigned int dest = 0;
            unsigned int gateway = 0;
            unsigned int flags = 0;
            int refcnt = 0;
            int use = 0;
            int metric = 0;
            unsigned int mask = 0;

            if (sscanf(line, "%s %x %x %x %d %d %d %x", iface, &dest, &gateway, &flags, &refcnt,
                       &use, &metric, &mask) >= 8) {
                char dest_str[24];
                char gw_str[24];
                char mask_str[24];
                char flag_str[8];

                format_hex_ip(dest, dest_str, sizeof(dest_str));
                format_hex_ip(gateway, gw_str, sizeof(gw_str));
                format_hex_ip(mask, mask_str, sizeof(mask_str));

                int f_idx = 0;
                if (flags & 0x0001) {
                    flag_str[f_idx++] = 'U';
                }
                if (flags & 0x0002) {
                    flag_str[f_idx++] = 'G';
                }
                if (flags & 0x0004) {
                    flag_str[f_idx++] = 'H';
                }
                flag_str[f_idx] = '\0';

                printf("%-15s %-15s %-15s %-5s %-6d %-6d %-6d %s\n", dest_str, gw_str, mask_str,
                       flag_str, metric, refcnt, use, iface);
            }
        }
        line = strtok(NULL, "\n");
    }

    return 0;
}
