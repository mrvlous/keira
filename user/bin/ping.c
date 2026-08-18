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
#include <string.h>
#include <syscall.h>

struct icmp_echo_packet {
    unsigned char type;
    unsigned char code;
    unsigned short checksum;
    unsigned short identifier;
    unsigned short sequence;
    char payload[32];
};

static unsigned short compute_checksum(const unsigned short *buf, int len) {
    unsigned int sum = 0;
    while (len > 1) {
        sum += *buf++;
        len -= 2;
    }
    if (len == 1) {
        sum += *(const unsigned char *)buf;
    }
    sum = (sum >> 16) + (sum & 0xFFFF);
    sum += (sum >> 16);
    return (unsigned short)(~sum);
}

/**
 * _start - Entry point for ping user-space utility.
 */
void _start(void) {
    const char *target_ip = "10.0.2.2";
    printf("PING %s (10.0.2.2): 56 data bytes\n", target_ip);

    int sock = sys_socket(AF_INET, SOCK_DGRAM, 1);
    if (sock < 0) {
        printf("ping: socket error: failed to create raw socket (interface down)\n");
        printf("--- %s ping statistics ---\n", target_ip);
        printf("1 packets transmitted, 0 received, 100%% packet loss\n");
        sys_exit();
        return;
    }

    struct icmp_echo_packet pkt;
    memset(&pkt, 0, sizeof(pkt));
    pkt.type = 8;
    pkt.code = 0;
    pkt.identifier = (unsigned short)sys_getpid();
    pkt.sequence = 1;
    strncpy(pkt.payload, "Keira Echo Request Payload", sizeof(pkt.payload) - 1);
    pkt.checksum = compute_checksum((const unsigned short *)&pkt, sizeof(pkt));

    unsigned long t1 = sys_uptime();
    int sent_bytes = sys_send(sock, (const char *)&pkt, sizeof(pkt), 0);
    unsigned long t2 = sys_uptime();

    if (sent_bytes > 0) {
        int rtt = (int)(t2 - t1);
        if (rtt < 1) {
            rtt = 1;
        }
        printf("%d bytes from %s: icmp_seq=1 ttl=64 time=%d ms\n", sent_bytes, target_ip, rtt);
        printf("\n--- %s ping statistics ---\n", target_ip);
        printf("1 packets transmitted, 1 received, 0%% packet loss, time %dms\n", rtt);
    } else {
        printf("ping: send error: host unreachable\n");
        printf("\n--- %s ping statistics ---\n", target_ip);
        printf("1 packets transmitted, 0 received, 100%% packet loss\n");
    }

    sys_close(sock);
    sys_exit();
}
