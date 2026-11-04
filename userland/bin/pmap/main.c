/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Process Memory Map Utility (pmap)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define PROC_MAPS_BUF_SIZE 4096

static void print_usage(void) {
    puts("Usage: pmap [OPTIONS] <PID> [PID...]");
    puts("");
    puts("Description:");
    puts("  Report virtual memory address space map of system processes.");
    puts("");
    puts("Options:");
    puts("  -x, --extended  Display detailed address mappings");
    puts("  -d, --device    Show device and inode mapping columns");
    puts("  -h, --help      Display this help reference and exit");
}

static uint64_t parse_hex(const char *s, char **endptr) {
    uint64_t val = 0;
    while (*s == ' ') {
        s++;
    }
    while ((*s >= '0' && *s <= '9') || (*s >= 'a' && *s <= 'f') || (*s >= 'A' && *s <= 'F')) {
        val <<= 4;
        if (*s >= '0' && *s <= '9') {
            val |= (*s - '0');
        } else if (*s >= 'a' && *s <= 'f') {
            val |= (*s - 'a' + 10);
        } else if (*s >= 'A' && *s <= 'F') {
            val |= (*s - 'A' + 10);
        }
        s++;
    }
    if (endptr) {
        *endptr = (char *)s;
    }
    return val;
}

static void dump_process_maps(int pid) {
    char path[64];
    char name[64] = "unknown";
    char buf[PROC_MAPS_BUF_SIZE];

    // Read cmdline to get process name
    snprintf(path, sizeof(path), "/proc/%d/cmdline", pid);
    int cfd = open(path, O_RDONLY, 0);
    if (cfd >= 0) {
        ssize_t cn = read(cfd, name, sizeof(name) - 1);
        if (cn > 0) {
            name[cn] = '\0';
        }
        close(cfd);
    }

    snprintf(path, sizeof(path), "/proc/%d/maps", pid);
    int fd = open(path, O_RDONLY, 0);
    if (fd < 0) {
        printf("pmap: cannot open %s\n", path);
        return;
    }

    ssize_t n = read(fd, buf, sizeof(buf) - 1);
    close(fd);

    if (n <= 0) {
        printf("%d: %s\n", pid, name);
        puts("  [no memory mappings found]");
        return;
    }
    buf[n] = '\0';

    printf("%d: %s\n", pid, name);
    printf("%-16s %7s %-5s %-16s %s\n", "Address", "Kbytes", "Mode", "Offset", "Mapping");

    uint64_t total_kb = 0;
    char *line = buf;

    while (*line) {
        char *eol = strchr(line, '\n');
        if (eol) {
            *eol = '\0';
        }

        // Format: <start>-<end> <perms> <offset> <dev> <inode> <pathname>
        char *p = line;
        while (*p == ' ') {
            p++;
        }
        if (*p) {
            char *end_ptr = NULL;
            uint64_t start_addr = parse_hex(p, &end_ptr);
            if (end_ptr && *end_ptr == '-') {
                p = end_ptr + 1;
                uint64_t end_addr = parse_hex(p, &end_ptr);
                p = end_ptr;
                while (*p == ' ') {
                    p++;
                }

                char perms[8] = "---";
                int pi = 0;
                while (*p && *p != ' ' && pi < 7) {
                    perms[pi++] = *p++;
                }
                perms[pi] = '\0';

                while (*p == ' ') {
                    p++;
                }
                uint64_t offset = parse_hex(p, &end_ptr);
                p = end_ptr;

                // Skip dev and inode
                while (*p == ' ') {
                    p++;
                }
                while (*p && *p != ' ') {
                    p++;
                }
                while (*p == ' ') {
                    p++;
                }
                while (*p && *p != ' ') {
                    p++;
                }
                while (*p == ' ') {
                    p++;
                }

                const char *mapping = (*p) ? p : "[anon]";
                uint64_t kbytes = (end_addr > start_addr) ? ((end_addr - start_addr) / 1024) : 0;
                total_kb += kbytes;

                char mode[4];
                mode[0] = (perms[0] != '-') ? perms[0] : '-';
                mode[1] = (perms[1] != '-') ? perms[1] : '-';
                mode[2] = (perms[2] != '-') ? perms[2] : '-';
                mode[3] = '\0';

                printf("%016llx %7llu %-5s %016llx %s\n",
                       (unsigned long long)start_addr,
                       (unsigned long long)kbytes,
                       mode,
                       (unsigned long long)offset,
                       mapping);
            }
        }

        if (!eol) {
            break;
        }
        line = eol + 1;
    }

    printf("---------------- -------\n");
    printf("total kB         %7llu\n", (unsigned long long)total_kb);
}

int main(int argc, char **argv) {
    int pids[32];
    int pid_count = 0;

    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        } else if (strcmp(argv[i], "-x") == 0 || strcmp(argv[i], "--extended") == 0 ||
                   strcmp(argv[i], "-d") == 0 || strcmp(argv[i], "--device") == 0) {
            continue;
        } else {
            int pid = atoi(argv[i]);
            if (pid >= 0 && pid_count < 32) {
                pids[pid_count++] = pid;
            }
        }
    }

    if (pid_count == 0) {
        dump_process_maps(1);
    } else {
        for (int i = 0; i < pid_count; i++) {
            dump_process_maps(pids[i]);
        }
    }

    return 0;
}

