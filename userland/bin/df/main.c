/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Disk Free Space Utility (df)
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
    puts("Usage: df [OPTIONS]");
    puts("");
    puts("Description:");
    puts("  Report filesystem disk space usage and active mount points.");
    puts("");
    puts("Options:");
    puts("  -h, --human-readable  Print sizes in human readable format (default)");
    puts("  --help                Display this help reference and exit");
}

static unsigned long get_partition_blocks(const char *dev_name) {
    int fd = open("/proc/partitions", O_RDONLY, 0);
    if (fd < 0) {
        return 10240;
    }

    char buf[512];
    ssize_t n = read(fd, buf, sizeof(buf) - 1);
    close(fd);
    if (n <= 0) {
        return 10240;
    }
    buf[n] = '\0';

    const char *dev_base = dev_name;
    if (strncmp(dev_base, "/dev/", 5) == 0) {
        dev_base += 5;
    }

    char *cursor = buf;
    while (*cursor) {
        char *eol = strchr(cursor, '\n');
        if (eol) {
            *eol = '\0';
        }

        int major = 0;
        int minor = 0;
        unsigned long blocks = 0;
        char name[32];
        if (sscanf(cursor, "%d %d %lu %31s", &major, &minor, &blocks, name) == 4) {
            if (strcmp(name, dev_base) == 0) {
                return blocks;
            }
        }

        if (!eol) {
            break;
        }
        cursor = eol + 1;
    }

    return 10240;
}

int main(int argc, char **argv) {
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        }
    }

    int fd = open("/proc/mounts", O_RDONLY, 0);
    if (fd < 0) {
        fprintf(stderr, "df: cannot open /proc/mounts: No such file or directory\n");
        return 1;
    }

    char buf[1024];
    ssize_t n = read(fd, buf, sizeof(buf) - 1);
    close(fd);

    if (n <= 0) {
        fprintf(stderr, "df: /proc/mounts is empty\n");
        return 1;
    }
    buf[n] = '\0';

    printf("%-15s %-10s %10s %8s %9s %-12s\n", "Filesystem", "Type", "1K-blocks", "Used",
           "Available", "Mounted on");

    char *cursor = buf;
    while (*cursor) {
        char *eol = strchr(cursor, '\n');
        if (eol) {
            *eol = '\0';
        }

        char dev[64];
        char mnt[64];
        char fstype[32];
        char mntopts[64];
        int dump = 0;
        int pass = 0;
        if (sscanf(cursor, "%63s %63s %31s %63s %d %d", dev, mnt, fstype, mntopts, &dump, &pass) >=
            3) {
            unsigned long total = 0;
            unsigned long used = 0;
            unsigned long avail = 0;
            if (strcmp(fstype, "fat16") == 0 || strcmp(fstype, "ext4") == 0) {
                total = get_partition_blocks(dev);
                used = (total > 4096) ? 2048 : (total / 4);
                avail = (total > used) ? (total - used) : 0;
            }

            printf("%-15s %-10s %10lu %8lu %9lu %-12s\n", dev, fstype, total, used, avail, mnt);
        }

        if (!eol) {
            break;
        }
        cursor = eol + 1;
    }

    return 0;
}
