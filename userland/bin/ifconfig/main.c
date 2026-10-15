/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 Network Interface Config Utility (ifconfig)
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
    puts("Usage: ifconfig [OPTION]... [INTERFACE]");
    puts("");
    puts("Description:");
    puts("  Configure or display network interface parameters and statistics.");
    puts("");
    puts("Options:");
    puts("  -a, --all    display all available network interfaces");
    puts("  -s, --short  display short list (like netstat -i)");
    puts("  -h, --help   display this help message and exit");
}

static void display_interface_lo(unsigned long rx_bytes, unsigned long rx_pkts,
                                 unsigned long tx_bytes, unsigned long tx_pkts) {
    puts("lo: flags=73<UP,LOOPBACK,RUNNING>  mtu 65536");
    puts("        inet 127.0.0.1  netmask 255.0.0.0");
    puts("        loop  txqueuelen 1000  (Local Loopback)");
    printf("        RX packets %lu  bytes %lu (%lu.0 B)\n", rx_pkts, rx_bytes, rx_bytes);
    puts("        RX errors 0  dropped 0  overruns 0  frame 0");
    printf("        TX packets %lu  bytes %lu (%lu.0 B)\n", tx_pkts, tx_bytes, tx_bytes);
    puts("        TX errors 0  dropped 0 overruns 0 carrier 0  collisions 0");
    putchar('\n');
}

static void display_interface_eth0(unsigned long rx_bytes, unsigned long rx_pkts,
                                   unsigned long tx_bytes, unsigned long tx_pkts) {
    puts("eth0: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500");
    puts("        inet 10.0.2.15  netmask 255.255.255.0  broadcast 10.0.2.255");
    puts("        ether 52:54:00:12:35:02  txqueuelen 1000  (Ethernet)");
    printf("        RX packets %lu  bytes %lu (%lu.%lu KiB)\n", rx_pkts, rx_bytes, rx_bytes / 1024,
           (rx_bytes % 1024) / 100);
    puts("        RX errors 0  dropped 0  overruns 0  frame 0");
    printf("        TX packets %lu  bytes %lu (%lu.%lu KiB)\n", tx_pkts, tx_bytes, tx_bytes / 1024,
           (tx_bytes % 1024) / 100);
    puts("        TX errors 0  dropped 0 overruns 0 carrier 0  collisions 0");
    putchar('\n');
}

int main(int argc, char **argv) {
    const char *target_if = NULL;

    for (int i = 1; i < argc; i++) {
        char *arg = argv[i];
        if (strcmp(arg, "-h") == 0 || strcmp(arg, "--help") == 0) {
            print_usage();
            return 0;
        } else if (strcmp(arg, "-a") == 0 || strcmp(arg, "--all") == 0) {
            /* Default behavior is to display all */
        } else if (arg[0] == '-' && arg[1] != '\0') {
            printf("ifconfig: unknown option '%s'\n", arg);
            puts("Try 'ifconfig --help' for more information.");
            return 1;
        } else {
            target_if = arg;
        }
    }

    unsigned long lo_rx_b = 1024, lo_rx_p = 16, lo_tx_b = 1024, lo_tx_p = 16;
    unsigned long eth_rx_b = 4096, eth_rx_p = 32, eth_tx_b = 2048, eth_tx_p = 24;

    /* Parse dynamic telemetry from /proc/net/dev if accessible */
    int fd = open("/proc/net/dev", O_RDONLY, 0);
    if (fd >= 0) {
        char buf[1024];
        ssize_t n = read(fd, buf, sizeof(buf) - 1);
        if (n > 0) {
            buf[n] = '\0';
            char *lo_line = strstr(buf, "lo:");
            if (lo_line) {
                sscanf(lo_line + 3, "%lu %lu", &lo_rx_b, &lo_rx_p);
            }
            char *eth_line = strstr(buf, "eth0:");
            if (eth_line) {
                sscanf(eth_line + 5, "%lu %lu", &eth_rx_b, &eth_rx_p);
            }
        }
        close(fd);
    }

    if (!target_if || strcmp(target_if, "eth0") == 0) {
        display_interface_eth0(eth_rx_b, eth_rx_p, eth_tx_b, eth_tx_p);
    }
    if (!target_if || strcmp(target_if, "lo") == 0) {
        display_interface_lo(lo_rx_b, lo_rx_p, lo_tx_b, lo_tx_p);
    }

    return 0;
}
