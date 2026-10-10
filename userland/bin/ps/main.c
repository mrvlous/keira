/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Process Status Utility (ps)
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

#define MAX_PID_SCAN 64
#define PROC_BUF_SIZE 512

static void print_usage(void) {
    puts("Usage: ps [OPTIONS]");
    puts("");
    puts("Description:");
    puts("  Report a snapshot of active system processes from /proc.");
    puts("");
    puts("Options:");
    puts("  -a, --all   Display all active process threads");
    puts("  -h, --help  Display this help reference and exit");
}

int main(int argc, char **argv) {
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage();
            return 0;
        }
    }

    printf("%5s %5s %-6s %s\n", "PID", "PPID", "STATE", "COMMAND");

    char path[32];
    char buf[PROC_BUF_SIZE];

    for (int pid = 0; pid < MAX_PID_SCAN; pid++) {
        snprintf(path, sizeof(path), "/proc/%d/status", pid);
        int fd = open(path, O_RDONLY, 0);
        if (fd < 0) {
            continue;
        }

        ssize_t n = read(fd, buf, sizeof(buf) - 1);
        close(fd);

        if (n <= 0) {
            continue;
        }
        buf[n] = '\0';

        char name[64] = "unknown";
        char state[32] = "?";
        int parsed_pid = pid;
        int parsed_ppid = 0;

        char *line = buf;
        while (line && *line) {
            char *next = strchr(line, '\n');
            if (next) {
                *next = '\0';
                next++;
            }

            if (strncmp(line, "Name:\t", 6) == 0) {
                strncpy(name, line + 6, sizeof(name) - 1);
                name[sizeof(name) - 1] = '\0';
            } else if (strncmp(line, "State:\t", 7) == 0) {
                strncpy(state, line + 7, sizeof(state) - 1);
                state[sizeof(state) - 1] = '\0';
                char *spc = strchr(state, ' ');
                if (spc) {
                    *spc = '\0';
                }
            } else if (strncmp(line, "Pid:\t", 5) == 0) {
                parsed_pid = atoi(line + 5);
            } else if (strncmp(line, "PPid:\t", 6) == 0) {
                parsed_ppid = atoi(line + 6);
            }

            line = next;
        }

        printf("%5d %5d %-6s %s\n", parsed_pid, parsed_ppid, state, name);
    }

    return 0;
}
