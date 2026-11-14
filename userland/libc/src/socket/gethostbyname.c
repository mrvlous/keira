// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
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

#include <arpa/inet.h>
#include <ctype.h>
#include <fcntl.h>
#include <netdb.h>
#include <string.h>
#include <sys/syscall.h>
#include <unistd.h>

int h_errno = 0;

static struct hostent s_hostent;
static char s_name[128];
static struct in_addr s_addr;
static char *s_addr_ptrs[2];
static char *s_aliases[1];

static int lookup_hosts_file(const char *name, struct in_addr *out_addr) {
    int fd = open("/etc/hosts", O_RDONLY, 0);
    if (fd < 0) {
        return 0;
    }

    char buf[1024];
    ssize_t n = read(fd, buf, sizeof(buf) - 1);
    close(fd);

    if (n <= 0) {
        return 0;
    }
    buf[n] = '\0';

    char *line = buf;
    while (*line) {
        while (*line == ' ' || *line == '\t') {
            line++;
        }
        if (*line == '#' || *line == '\n' || *line == '\r') {
            while (*line && *line != '\n') {
                line++;
            }
            if (*line == '\n') {
                line++;
            }
            continue;
        }

        char *line_end = strchr(line, '\n');
        if (line_end) {
            *line_end = '\0';
        }

        char ip_str[64];
        char host_str[128];
        ip_str[0] = '\0';
        host_str[0] = '\0';

        char *p = line;
        size_t ip_len = 0;
        while (*p && !isspace((unsigned char)*p) && ip_len < sizeof(ip_str) - 1) {
            ip_str[ip_len++] = *p++;
        }
        ip_str[ip_len] = '\0';

        while (*p) {
            while (*p && isspace((unsigned char)*p)) {
                p++;
            }
            if (!*p || *p == '#') {
                break;
            }

            size_t h_len = 0;
            while (*p && !isspace((unsigned char)*p) && h_len < sizeof(host_str) - 1) {
                host_str[h_len++] = *p++;
            }
            host_str[h_len] = '\0';

            if (strcmp(host_str, name) == 0) {
                if (inet_aton(ip_str, out_addr)) {
                    return 1;
                }
            }
        }

        if (!line_end) {
            break;
        }
        line = line_end + 1;
    }

    return 0;
}

struct hostent *gethostbyname(const char *name) {
    if (!name || !*name) {
        h_errno = HOST_NOT_FOUND;
        return NULL;
    }

    if (inet_aton(name, &s_addr)) {
        strncpy(s_name, name, sizeof(s_name) - 1);
        s_name[sizeof(s_name) - 1] = '\0';
        s_addr_ptrs[0] = (char *)&s_addr;
        s_addr_ptrs[1] = NULL;
        s_aliases[0] = NULL;
        s_hostent.h_name = s_name;
        s_hostent.h_aliases = s_aliases;
        s_hostent.h_addrtype = AF_INET;
        s_hostent.h_length = sizeof(struct in_addr);
        s_hostent.h_addr_list = s_addr_ptrs;
        return &s_hostent;
    }

    if (lookup_hosts_file(name, &s_addr)) {
        strncpy(s_name, name, sizeof(s_name) - 1);
        s_name[sizeof(s_name) - 1] = '\0';
        s_addr_ptrs[0] = (char *)&s_addr;
        s_addr_ptrs[1] = NULL;
        s_aliases[0] = NULL;
        s_hostent.h_name = s_name;
        s_hostent.h_aliases = s_aliases;
        s_hostent.h_addrtype = AF_INET;
        s_hostent.h_length = sizeof(struct in_addr);
        s_hostent.h_addr_list = s_addr_ptrs;
        return &s_hostent;
    }

    int res = sys_dns_resolve(name, (uint8_t *)&s_addr.s_addr);
    if (res == 0) {
        strncpy(s_name, name, sizeof(s_name) - 1);
        s_name[sizeof(s_name) - 1] = '\0';
        s_addr_ptrs[0] = (char *)&s_addr;
        s_addr_ptrs[1] = NULL;
        s_aliases[0] = NULL;
        s_hostent.h_name = s_name;
        s_hostent.h_aliases = s_aliases;
        s_hostent.h_addrtype = AF_INET;
        s_hostent.h_length = sizeof(struct in_addr);
        s_hostent.h_addr_list = s_addr_ptrs;
        return &s_hostent;
    }

    h_errno = HOST_NOT_FOUND;
    return NULL;
}
