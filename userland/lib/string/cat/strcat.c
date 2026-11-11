// SPDX-License-Identifier: GPL-2.0-only

#include <stddef.h>
#include <string.h>

char *strcat(char *dest, const char *src) {
    char *ret = dest;
    while (*dest)
        dest++;
    while ((*dest++ = *src++))
        ;
    return ret;
}

char *strncat(char *dest, const char *src, size_t n) {
    char *ret = dest;
    while (*dest)
        dest++;
    while (n && (*dest++ = *src++))
        n--;
    if (!n)
        *dest = '\0';
    return ret;
}
