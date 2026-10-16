/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 ARP Table Utility (arp)
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
    puts("Usage: arp [OPTIONS]");
    puts("");
    puts("Description:");
    puts("  Display or manipulate the kernel ARP neighbor cache.");
    puts("");
    puts("Options:");
    puts("  -a, --all      display all entries in the cache");
    puts("  -n, --numeric  do not resolve hostnames");
    puts("  -h, --help     display this help message and exit");
}

int main(int argc, char **argv) {
    for (int i = 1; i < argc; i++) {
        char *arg = argv[i];
        if (strcmp(arg, "-h") == 0 || strcmp(arg, "--help") == 0) {
            print_usage();
            return 0;
        }
        if (strcmp(arg, "-a") == 0 || strcmp(arg, "--all") == 0 || strcmp(arg, "-n") == 0 ||
            strcmp(arg, "--numeric") == 0) {
            continue;
        }
    }

    int fd = open("/proc/net/arp", O_RDONLY, 0);
    if (fd < 0) {
        fprintf(stderr, "arp: cannot open /proc/net/arp: No such file or directory\n");
        return 1;
    }

    char buf[1024];
    ssize_t n = read(fd, buf, sizeof(buf) - 1);
    close(fd);

    if (n <= 0) {
        fprintf(stderr, "arp: /proc/net/arp empty or read error\n");
        return 1;
    }
    buf[n] = '\0';

    puts("Address                  HWtype  HWaddress           Flags Mask            Iface");

    char *line = strtok(buf, "\n");
    int line_num = 0;
    while (line) {
        line_num++;
        if (line_num > 1) {
            char ip[32];
            unsigned int hw_type = 0;
            unsigned int flags = 0;
            char mac[32];
            char mask[32];
            char iface[32];

            if (sscanf(line, "%s %x %x %s %s %s", ip, &hw_type, &flags, mac, mask, iface) >= 6) {
                const char *hw_str = (hw_type == 1) ? "ether" : "unspec";
                const char *flag_str = (flags & 0x2) ? "C" : "";

                printf("%-24s %-7s %-19s %-5s %-15s %s\n", ip, hw_str, mac, flag_str, mask, iface);
            }
        }
        line = strtok(NULL, "\n");
    }

    return 0;
}
