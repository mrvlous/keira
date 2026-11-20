// SPDX-License-Identifier: GPL-2.0-only
/*
 * Keira Kernel - Freestanding Module Insertion Utility (insmod)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
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

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/syscall.h>
#include <unistd.h>

static void print_usage(void) {
    puts("Usage: insmod <module_name> [size]");
    puts("");
    puts("Description:");
    puts("  Insert a loadable module into the Linux-grade kernel.");
    puts("");
    puts("Options:");
    puts("  -h, --help  Display this help reference and exit");
}

int main(int argc, char **argv) {
    if (argc < 2) {
        print_usage();
        return 1;
    }

    if (strcmp(argv[1], "-h") == 0 || strcmp(argv[1], "--help") == 0) {
        print_usage();
        return 0;
    }

    const char *name = argv[1];
    size_t size = 32768;
    if (argc >= 3) {
        size = (size_t)atoi(argv[2]);
        if (size == 0) {
            size = 32768;
        }
    }

    int64_t ret = syscall2(SYS_INIT_MODULE, (uint64_t)(uintptr_t)name, (uint64_t)size);
    if (ret < 0) {
        fprintf(stderr, "insmod: ERROR: could not insert module '%s': error %ld\n", name, (long)ret);
        return 1;
    }

    printf("insmod: module '%s' inserted successfully at slot %ld\n", name, (long)ret);
    return 0;
}
