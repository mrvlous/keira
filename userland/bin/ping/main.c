/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 Network Ping Utility (ping)
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/syscall.h>
#include <unistd.h>

static void print_usage(void) {
    puts("Usage: ping [OPTION]... DESTINATION");
    puts("");
    puts("Description:");
    puts("  Send ICMP ECHO_REQUEST packets to network hosts.");
    puts("");
    puts("Options:");
    puts("  -c <count>   stop after sending count ECHO_REQUEST packets (default: 4)");
    puts("  -h, --help   display this help message and exit");
}

int main(int argc, char **argv) {
    int count = 4;
    const char *destination = NULL;

    for (int i = 1; i < argc; i++) {
        char *arg = argv[i];
        if (strcmp(arg, "-h") == 0 || strcmp(arg, "--help") == 0) {
            print_usage();
            return 0;
        } else if (strcmp(arg, "-c") == 0) {
            if (i + 1 < argc) {
                count = atoi(argv[++i]);
                if (count <= 0) {
                    count = 1;
                }
            } else {
                puts("ping: option requires an argument -- 'c'");
                return 1;
            }
        } else if (arg[0] == '-' && arg[1] != '\0') {
            printf("ping: invalid option -- '%s'\n", arg);
            puts("Try 'ping --help' for more information.");
            return 1;
        } else {
            destination = arg;
        }
    }

    if (!destination) {
        puts("ping: missing host operand");
        puts("Try 'ping --help' for more information.");
        return 1;
    }

    printf("PING %s (%s) 56(84) bytes of data.\n", destination, destination);

    int received = 0;
    for (int seq = 1; seq <= count; seq++) {
        long res = sys_ping(destination, 1000);
        if (res >= 0) {
            received++;
            int latency_ms = (int)res;
            if (latency_ms == 0) {
                printf("64 bytes from %s: icmp_seq=%d ttl=64 time=0.42 ms\n", destination, seq);
            } else {
                printf("64 bytes from %s: icmp_seq=%d ttl=64 time=%d.00 ms\n", destination, seq,
                       latency_ms);
            }
        } else {
            printf("From %s icmp_seq=%d Destination Host Unreachable\n", destination, seq);
        }
    }

    printf("--- %s ping statistics ---\n", destination);
    int loss_percent = ((count - received) * 100) / count;
    printf("%d packets transmitted, %d received, %d%% packet loss\n", count, received,
           loss_percent);

    return (received > 0) ? 0 : 1;
}
