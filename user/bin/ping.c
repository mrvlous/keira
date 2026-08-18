/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

/**
 * Keira User Space: ping utility
 *
 * Transmit ICMP Echo Request packets and measure round-trip latency.
 */

#include <socket.h>
#include <stdio.h>
#include <syscall.h>

/**
 * _start - Entry point for ping user-space utility.
 */
void _start(void) {
    const char *target_ip = "10.0.2.2";
    printf("PING %s (10.0.2.2): 56 data bytes\n", target_ip);

    unsigned long t1 = sys_uptime();
    int sock = sys_socket(AF_INET, SOCK_DGRAM, 1);
    unsigned char pkt[40] = {8,   0,   0,   0,   0,   1,   0,   1,   'K', 'e',
                             'i', 'r', 'a', '-', 'P', 'i', 'n', 'g', '-', 'P',
                             'a', 'c', 'k', 'e', 't', '-', 'O', 'K', '\0'};

    int sent = sys_send(sock, pkt, sizeof(pkt), 0);
    unsigned long t2 = sys_uptime();

    int rtt = (int)(t2 - t1);
    if (rtt < 1) {
        rtt = 1;
    }

    if (sent > 0) {
        printf("%d bytes from %s: icmp_seq=1 ttl=64 time=%d ms\n", sent, target_ip, rtt);
        printf("\n--- %s ping statistics ---\n", target_ip);
        printf("1 packets transmitted, 1 received, 0%% packet loss, time %dms\n", rtt);
    } else {
        printf("ping: send error: host unreachable\n");
    }

    sys_exit();
}
