// SPDX-License-Identifier: GPL-2.0-only

#include <arpa/inet.h>
#include <stdio.h>
#include <string.h>

uint16_t htons(uint16_t hostshort) {
    return (uint16_t)(((hostshort & 0xFF) << 8) | ((hostshort >> 8) & 0xFF));
}

uint16_t ntohs(uint16_t netshort) {
    return htons(netshort);
}

uint32_t htonl(uint32_t hostlong) {
    return ((hostlong & 0x000000FF) << 24) | ((hostlong & 0x0000FF00) << 8) |
           ((hostlong & 0x00FF0000) >> 8) | ((hostlong & 0xFF000000) >> 24);
}

uint32_t ntohl(uint32_t netlong) {
    return htonl(netlong);
}

int inet_aton(const char *cp, struct in_addr *inp) {
    if (!cp || !inp) {
        return 0;
    }

    unsigned int b0 = 0;
    unsigned int b1 = 0;
    unsigned int b2 = 0;
    unsigned int b3 = 0;
    char trailing = '\0';

    int matched = sscanf(cp, "%u.%u.%u.%u%c", &b0, &b1, &b2, &b3, &trailing);
    if (matched != 4) {
        return 0;
    }

    if (b0 > 255 || b1 > 255 || b2 > 255 || b3 > 255) {
        return 0;
    }

    uint8_t bytes[4];
    bytes[0] = (uint8_t)b0;
    bytes[1] = (uint8_t)b1;
    bytes[2] = (uint8_t)b2;
    bytes[3] = (uint8_t)b3;

    memcpy(&inp->s_addr, bytes, 4);
    return 1;
}

in_addr_t inet_addr(const char *cp) {
    struct in_addr addr;
    if (inet_aton(cp, &addr)) {
        return addr.s_addr;
    }
    return (in_addr_t)0xFFFFFFFF;
}

char *inet_ntoa(struct in_addr in) {
    static char buf[32];
    const uint8_t *b = (const uint8_t *)&in.s_addr;
    snprintf(buf, sizeof(buf), "%u.%u.%u.%u", b[0], b[1], b[2], b[3]);
    return buf;
}
