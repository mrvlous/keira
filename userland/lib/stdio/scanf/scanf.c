/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>

int vsscanf(const char *str, const char *format, va_list ap) {
    if (!str || !format)
        return -1;
    int matched = 0;
    while (*format) {
        if (isspace((unsigned char)*format)) {
            while (isspace((unsigned char)*format))
                format++;
            while (isspace((unsigned char)*str))
                str++;
            continue;
        }
        if (*format != '%') {
            if (*str != *format)
                break;
            str++;
            format++;
            continue;
        }
        format++;
        if (*format == '%') {
            if (*str != '%')
                break;
            str++;
            format++;
            continue;
        }

        if (*format == 'd' || *format == 'i') {
            while (isspace((unsigned char)*str))
                str++;
            if (!*str)
                break;
            char *endptr = NULL;
            long val = strtol(str, &endptr, (*format == 'i') ? 0 : 10);
            if (endptr == str)
                break;
            int *out = va_arg(ap, int *);
            if (out)
                *out = (int)val;
            str = endptr;
            matched++;
            format++;
        } else if (*format == 'u') {
            while (isspace((unsigned char)*str))
                str++;
            if (!*str)
                break;
            char *endptr = NULL;
            unsigned long val = strtoul(str, &endptr, 10);
            if (endptr == str)
                break;
            unsigned int *out = va_arg(ap, unsigned int *);
            if (out)
                *out = (unsigned int)val;
            str = endptr;
            matched++;
            format++;
        } else if (*format == 'x' || *format == 'X') {
            while (isspace((unsigned char)*str))
                str++;
            if (!*str)
                break;
            char *endptr = NULL;
            unsigned long val = strtoul(str, &endptr, 16);
            if (endptr == str)
                break;
            unsigned int *out = va_arg(ap, unsigned int *);
            if (out)
                *out = (unsigned int)val;
            str = endptr;
            matched++;
            format++;
        } else if (*format == 's') {
            while (isspace((unsigned char)*str))
                str++;
            if (!*str)
                break;
            char *out = va_arg(ap, char *);
            if (!out)
                break;
            int s_idx = 0;
            while (*str && !isspace((unsigned char)*str)) {
                out[s_idx++] = *str++;
            }
            out[s_idx] = '\0';
            matched++;
            format++;
        } else if (*format == 'c') {
            if (!*str)
                break;
            char *out = va_arg(ap, char *);
            if (out)
                *out = *str++;
            matched++;
            format++;
        } else {
            break;
        }
    }
    return matched;
}

int sscanf(const char *str, const char *format, ...) {
    va_list ap;
    va_start(ap, format);
    int ret = vsscanf(str, format, ap);
    va_end(ap);
    return ret;
}
