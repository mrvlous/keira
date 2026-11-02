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
#include <sys/syscall.h>
#include <unistd.h>

#define SYSLOG_ACTION_READ_ALL 3
#define DMESG_BUF_SIZE 32768

static char klog_buffer[DMESG_BUF_SIZE];

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

    /* Primary: query live kernel ring buffer via Syscall 44 (SYS_SYSLOG) */
    long n = syscall(SYS_SYSLOG, (long)SYSLOG_ACTION_READ_ALL, (long)klog_buffer,
                     (long)sizeof(klog_buffer));
    if (n > 0) {
        write(STDOUT_FILENO, klog_buffer, (size_t)n);
        return 0;
    }

    /* Fallback: inspect legacy log files if present */
    const char *log_files[] = {"/var/log/boot.log", "/var/log/system.log"};
    int printed = 0;

    for (size_t f = 0; f < sizeof(log_files) / sizeof(log_files[0]); f++) {
        int fd = open(log_files[f], O_RDONLY, 0);
        if (fd >= 0) {
            char buf[1024];
            ssize_t bytes_read;
            while ((bytes_read = read(fd, buf, sizeof(buf))) > 0) {
                write(STDOUT_FILENO, buf, (size_t)bytes_read);
                printed = 1;
            }
            close(fd);
        }
    }

    if (!printed) {
        fprintf(stderr, "dmesg: no active kernel log records available\n");
        return 1;
    }

    return 0;
}
