/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 System Information Utility (uname)
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
    puts("Usage: uname [OPTION]...");
    puts("");
    puts("Description:");
    puts("  Print certain system information. With no OPTION, same as -s.");
    puts("");
    puts("Options:");
    puts("  -a, --all                print all information");
    puts("  -s, --kernel-name        print the kernel name");
    puts("  -n, --nodename           print the network node hostname");
    puts("  -r, --kernel-release     print the kernel release");
    puts("  -v, --kernel-version     print the kernel version");
    puts("  -m, --machine            print the machine hardware name");
    puts("  -h, --help               display this help message and exit");
}

int main(int argc, char **argv) {
    int opt_a = 0, opt_s = 0, opt_n = 0, opt_r = 0, opt_v = 0, opt_m = 0;

    for (int i = 1; i < argc; i++) {
        char *arg = argv[i];
        if (strcmp(arg, "-h") == 0 || strcmp(arg, "--help") == 0) {
            print_usage();
            return 0;
        } else if (strcmp(arg, "-a") == 0 || strcmp(arg, "--all") == 0) {
            opt_a = 1;
        } else if (strcmp(arg, "-s") == 0 || strcmp(arg, "--kernel-name") == 0) {
            opt_s = 1;
        } else if (strcmp(arg, "-n") == 0 || strcmp(arg, "--nodename") == 0) {
            opt_n = 1;
        } else if (strcmp(arg, "-r") == 0 || strcmp(arg, "--kernel-release") == 0) {
            opt_r = 1;
        } else if (strcmp(arg, "-v") == 0 || strcmp(arg, "--kernel-version") == 0) {
            opt_v = 1;
        } else if (strcmp(arg, "-m") == 0 || strcmp(arg, "--machine") == 0) {
            opt_m = 1;
        } else if (arg[0] == '-' && arg[1] != '\0') {
            for (size_t j = 1; j < strlen(arg); j++) {
                if (arg[j] == 'a')
                    opt_a = 1;
                else if (arg[j] == 's')
                    opt_s = 1;
                else if (arg[j] == 'n')
                    opt_n = 1;
                else if (arg[j] == 'r')
                    opt_r = 1;
                else if (arg[j] == 'v')
                    opt_v = 1;
                else if (arg[j] == 'm')
                    opt_m = 1;
                else {
                    printf("uname: invalid option -- '%c'\n", arg[j]);
                    puts("Try 'uname --help' for more information.");
                    return 1;
                }
            }
        }
    }

    if (opt_a) {
        opt_s = opt_n = opt_r = opt_v = opt_m = 1;
    } else if (!opt_s && !opt_n && !opt_r && !opt_v && !opt_m) {
        opt_s = 1;
    }

    const char *kernel_name = "Keira";
    char hostname[64] = "keira";
    char release[32] = "0.7.0";
    char version[64] = "#1 SMP 2026";
#if defined(__x86_64__)
    const char *machine = "x86_64";
#else
    const char *machine = "i686";
#endif

    int vfd = open("/proc/version", O_RDONLY, 0);
    if (vfd >= 0) {
        char vbuf[256];
        ssize_t vn = read(vfd, vbuf, sizeof(vbuf) - 1);
        close(vfd);
        if (vn > 0) {
            vbuf[vn] = '\0';
            char *ver_pos = strstr(vbuf, "version ");
            if (ver_pos) {
                ver_pos += 8;
                char *space = strchr(ver_pos, ' ');
                if (space) {
                    size_t rlen = (size_t)(space - ver_pos);
                    if (rlen < sizeof(release)) {
                        memcpy(release, ver_pos, rlen);
                        release[rlen] = '\0';
                    }
                }
            }
            char *hash_pos = strchr(vbuf, '#');
            if (hash_pos) {
                char *gcc_pos = strstr(hash_pos, " gcc");
                if (gcc_pos) {
                    size_t vlen = (size_t)(gcc_pos - hash_pos);
                    if (vlen < sizeof(version)) {
                        memcpy(version, hash_pos, vlen);
                        version[vlen] = '\0';
                    }
                }
            }
        }
    }

    int fd = open("/etc/hostname", O_RDONLY, 0);
    if (fd >= 0) {
        ssize_t n = read(fd, hostname, sizeof(hostname) - 1);
        if (n > 0) {
            while (n > 0 &&
                   (hostname[n - 1] == '\n' || hostname[n - 1] == '\r' || hostname[n - 1] == ' ')) {
                n--;
            }
            hostname[n] = '\0';
        }
        close(fd);
    }

    int first = 1;
    if (opt_s) {
        printf("%s%s", first ? "" : " ", kernel_name);
        first = 0;
    }
    if (opt_n) {
        printf("%s%s", first ? "" : " ", hostname);
        first = 0;
    }
    if (opt_r) {
        printf("%s%s", first ? "" : " ", release);
        first = 0;
    }
    if (opt_v) {
        printf("%s%s", first ? "" : " ", version);
        first = 0;
    }
    if (opt_m) {
        printf("%s%s", first ? "" : " ", machine);
    }
    putchar('\n');

    return 0;
}
