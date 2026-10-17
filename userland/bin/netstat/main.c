/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Standalone Ring 3 Network Statistics Utility (netstat)
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
    puts("Usage: netstat [OPTIONS]");
    puts("");
    puts("Description:");
    puts("  Display active network connections, sockets, and statistics.");
    puts("");
    puts("Options:");
    puts("  -a, --all         display all sockets (listening and non-listening)");
    puts("  -t, --tcp         display only TCP sockets");
    puts("  -u, --udp         display only UDP sockets");
    puts("  -w, --raw         display only RAW sockets");
    puts("  -l, --listening   display only listening sockets");
    puts("  -r, --route       display kernel routing table");
    puts("  -s, --statistics  display socket allocation statistics");
    puts("  -n, --numeric     do not resolve names (default)");
    puts("  -h, --help        display this help manual and exit");
}

static void format_addr(unsigned int ip_hex, unsigned int port_hex, char *out, size_t size) {
    unsigned char b0 = (unsigned char)(ip_hex & 0xFF);
    unsigned char b1 = (unsigned char)((ip_hex >> 8) & 0xFF);
    unsigned char b2 = (unsigned char)((ip_hex >> 16) & 0xFF);
    unsigned char b3 = (unsigned char)((ip_hex >> 24) & 0xFF);

    if (port_hex == 0 && ip_hex == 0) {
        snprintf(out, size, "0.0.0.0:*");
    } else if (port_hex == 0) {
        snprintf(out, size, "%u.%u.%u.%u:*", b0, b1, b2, b3);
    } else {
        snprintf(out, size, "%u.%u.%u.%u:%u", b0, b1, b2, b3, port_hex);
    }
}

static const char *tcp_state_str(unsigned int st) {
    switch (st) {
    case 0x01:
        return "ESTABLISHED";
    case 0x02:
        return "SYN_SENT";
    case 0x03:
        return "SYN_RECV";
    case 0x04:
        return "FIN_WAIT1";
    case 0x05:
        return "FIN_WAIT2";
    case 0x06:
        return "TIME_WAIT";
    case 0x07:
        return "CLOSE";
    case 0x08:
        return "CLOSE_WAIT";
    case 0x09:
        return "LAST_ACK";
    case 0x0A:
        return "LISTEN";
    case 0x0B:
        return "CLOSING";
    default:
        return "UNKNOWN";
    }
}

static void dump_sock_file(const char *proto, const char *path, int only_listening) {
    int fd = open(path, O_RDONLY, 0);
    if (fd < 0) {
        return;
    }

    char buf[2048];
    ssize_t n = read(fd, buf, sizeof(buf) - 1);
    close(fd);
    if (n <= 0) {
        return;
    }
    buf[n] = '\0';

    char *line = strtok(buf, "\n");
    int line_num = 0;
    while (line) {
        line_num++;
        if (line_num > 1) {
            int slot = 0;
            unsigned int loc_ip = 0, loc_port = 0;
            unsigned int rem_ip = 0, rem_port = 0;
            unsigned int st = 0;
            unsigned int tx_q = 0, rx_q = 0;

            if (sscanf(line, " %d: %x:%x %x:%x %x %x:%x", &slot, &loc_ip, &loc_port, &rem_ip,
                       &rem_port, &st, &tx_q, &rx_q) >= 7) {
                if (only_listening && st != 0x0A && strcmp(proto, "tcp") == 0) {
                    line = strtok(NULL, "\n");
                    continue;
                }

                char local[32];
                char foreign[32];
                format_addr(loc_ip, loc_port, local, sizeof(local));
                format_addr(rem_ip, rem_port, foreign, sizeof(foreign));

                const char *st_str = "";
                if (strcmp(proto, "tcp") == 0) {
                    st_str = tcp_state_str(st);
                }

                printf("%-5s %6u %6u %-23s %-23s %-11s\n", proto, rx_q, tx_q, local, foreign,
                       st_str);
            }
        }
        line = strtok(NULL, "\n");
    }
}

static void show_routing_table(void) {
    int fd = open("/proc/net/route", O_RDONLY, 0);
    if (fd < 0) {
        fprintf(stderr, "netstat: cannot open /proc/net/route\n");
        return;
    }

    char buf[1024];
    ssize_t n = read(fd, buf, sizeof(buf) - 1);
    close(fd);
    if (n <= 0) {
        return;
    }
    buf[n] = '\0';

    puts("Kernel IP routing table");
    puts("Destination     Gateway         Genmask         Flags Metric Ref    Use Iface");

    char *line = strtok(buf, "\n");
    int line_num = 0;
    while (line) {
        line_num++;
        if (line_num > 1) {
            char iface[32];
            unsigned int dest = 0, gateway = 0, flags = 0, mask = 0;
            int refcnt = 0, use = 0, metric = 0;

            if (sscanf(line, "%s %x %x %x %d %d %d %x", iface, &dest, &gateway, &flags, &refcnt,
                       &use, &metric, &mask) >= 8) {
                char dest_str[24], gw_str[24], mask_str[24], flag_str[8];

                unsigned char d0 = (unsigned char)(dest & 0xFF);
                unsigned char d1 = (unsigned char)((dest >> 8) & 0xFF);
                unsigned char d2 = (unsigned char)((dest >> 16) & 0xFF);
                unsigned char d3 = (unsigned char)((dest >> 24) & 0xFF);
                snprintf(dest_str, sizeof(dest_str), "%u.%u.%u.%u", d0, d1, d2, d3);

                unsigned char g0 = (unsigned char)(gateway & 0xFF);
                unsigned char g1 = (unsigned char)((gateway >> 8) & 0xFF);
                unsigned char g2 = (unsigned char)((gateway >> 16) & 0xFF);
                unsigned char g3 = (unsigned char)((gateway >> 24) & 0xFF);
                snprintf(gw_str, sizeof(gw_str), "%u.%u.%u.%u", g0, g1, g2, g3);

                unsigned char m0 = (unsigned char)(mask & 0xFF);
                unsigned char m1 = (unsigned char)((mask >> 8) & 0xFF);
                unsigned char m2 = (unsigned char)((mask >> 16) & 0xFF);
                unsigned char m3 = (unsigned char)((mask >> 24) & 0xFF);
                snprintf(mask_str, sizeof(mask_str), "%u.%u.%u.%u", m0, m1, m2, m3);

                int f_idx = 0;
                if (flags & 0x0001)
                    flag_str[f_idx++] = 'U';
                if (flags & 0x0002)
                    flag_str[f_idx++] = 'G';
                if (flags & 0x0004)
                    flag_str[f_idx++] = 'H';
                flag_str[f_idx] = '\0';

                printf("%-15s %-15s %-15s %-5s %-6d %-6d %-6d %s\n", dest_str, gw_str, mask_str,
                       flag_str, metric, refcnt, use, iface);
            }
        }
        line = strtok(NULL, "\n");
    }
}

static void show_statistics(void) {
    int fd = open("/proc/net/sockstat", O_RDONLY, 0);
    if (fd < 0) {
        fprintf(stderr, "netstat: cannot open /proc/net/sockstat\n");
        return;
    }

    char buf[1024];
    ssize_t n = read(fd, buf, sizeof(buf) - 1);
    close(fd);
    if (n > 0) {
        buf[n] = '\0';
        printf("%s", buf);
    }
}

int main(int argc, char **argv) {
    int show_all = 0;
    int show_tcp = 0;
    int show_udp = 0;
    int show_raw = 0;
    int only_listening = 0;
    int show_route = 0;
    int show_stats = 0;

    for (int i = 1; i < argc; i++) {
        char *arg = argv[i];
        if (strcmp(arg, "-h") == 0 || strcmp(arg, "--help") == 0) {
            print_usage();
            return 0;
        }
        if (strcmp(arg, "-a") == 0 || strcmp(arg, "--all") == 0) {
            show_all = 1;
        } else if (strcmp(arg, "-t") == 0 || strcmp(arg, "--tcp") == 0) {
            show_tcp = 1;
        } else if (strcmp(arg, "-u") == 0 || strcmp(arg, "--udp") == 0) {
            show_udp = 1;
        } else if (strcmp(arg, "-w") == 0 || strcmp(arg, "--raw") == 0) {
            show_raw = 1;
        } else if (strcmp(arg, "-l") == 0 || strcmp(arg, "--listening") == 0) {
            only_listening = 1;
        } else if (strcmp(arg, "-r") == 0 || strcmp(arg, "--route") == 0) {
            show_route = 1;
        } else if (strcmp(arg, "-s") == 0 || strcmp(arg, "--statistics") == 0) {
            show_stats = 1;
        } else if (strcmp(arg, "-n") == 0 || strcmp(arg, "--numeric") == 0) {
            /* Numeric is default */
        } else if (strcmp(arg, "-tuln") == 0) {
            show_tcp = 1;
            show_udp = 1;
            only_listening = 1;
        }
    }

    if (show_route) {
        show_routing_table();
        return 0;
    }

    if (show_stats) {
        show_statistics();
        return 0;
    }

    if (show_all || (!show_tcp && !show_udp && !show_raw)) {
        show_tcp = 1;
        show_udp = 1;
        show_raw = 1;
    }

    puts("Active Internet connections (servers and established)");
    puts("Proto Recv-Q Send-Q Local Address           Foreign Address         State");

    if (show_tcp) {
        dump_sock_file("tcp", "/proc/net/tcp", only_listening);
    }
    if (show_udp) {
        dump_sock_file("udp", "/proc/net/udp", only_listening);
    }
    if (show_raw) {
        dump_sock_file("raw", "/proc/net/raw", only_listening);
    }

    return 0;
}
