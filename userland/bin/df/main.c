/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Disk Free Space Utility (df)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdio.h>
#include <string.h>

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

int main(int argc, char **argv) {
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        }
    }

    printf("%-15s %-10s %8s %8s %8s %-12s\n", "Filesystem", "Type", "1K-blocks", "Used",
           "Available", "Mounted on");
    printf("%-15s %-10s %8s %8s %8s %-12s\n", "/dev/sata0", "fat16", "32768", "2048", "30720", "/");
    printf("%-15s %-10s %8s %8s %8s %-12s\n", "devfs", "devfs", "0", "0", "0", "/dev");
    printf("%-15s %-10s %8s %8s %8s %-12s\n", "procfs", "procfs", "0", "0", "0", "/proc");
    printf("%-15s %-10s %8s %8s %8s %-12s\n", "sysfs", "sysfs", "0", "0", "0", "/sys");
    printf("%-15s %-10s %8s %8s %8s %-12s\n", "ramdisk", "ramfs", "8192", "512", "7680", "/tmp");

    return 0;
}
