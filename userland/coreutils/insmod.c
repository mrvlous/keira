/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Module Insertion Utility (insmod)
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
#include <stdlib.h>
#include <string.h>
#include <sys/syscall.h>
#include <unistd.h>

static void print_usage(void) {
    puts("Usage: insmod <module_file.ko | module_name> [size]");
    puts("");
    puts("Description:");
    puts("  Insert an ELF relocatable module or dynamic driver into the kernel.");
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

    const char *target = argv[1];

    // Attempt to open and load as a relocatable ELF .ko object
    int fd = open(target, O_RDONLY, 0);
    if (fd >= 0) {
        static char elf_buf[65536];
        ssize_t n = read(fd, elf_buf, sizeof(elf_buf));
        close(fd);

        if (n >= 16 && (unsigned char)elf_buf[0] == 0x7F && elf_buf[1] == 'E' &&
            elf_buf[2] == 'L' && elf_buf[3] == 'F') {
            int64_t ret = syscall2(SYS_INIT_MODULE, (uint64_t)(uintptr_t)elf_buf, (uint64_t)n);
            if (ret < 0) {
                fprintf(stderr, "insmod: ERROR: could not insert ELF module '%s': error %ld\n",
                        target, (long)ret);
                return 1;
            }
            printf("insmod: ELF module '%s' inserted successfully\n", target);
            return 0;
        }
    }

    // Fallback to name-based registration
    const char *name = target;
    size_t size = 32768;
    if (argc >= 3) {
        size = (size_t)atoi(argv[2]);
        if (size == 0) {
            size = 32768;
        }
    }

    int64_t ret = syscall2(SYS_INIT_MODULE, (uint64_t)(uintptr_t)name, (uint64_t)size);
    if (ret < 0) {
        fprintf(stderr, "insmod: ERROR: could not insert module '%s': error %ld\n", name,
                (long)ret);
        return 1;
    }

    printf("insmod: module '%s' inserted successfully at slot %ld\n", name, (long)ret);
    return 0;
}
