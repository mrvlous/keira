/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdio.h>
#include <string.h>
#include <syscall.h>

int putchar(int c) {
    char ch = (char)c;
    sys_write(1, &ch, 1);
    return c;
}

int puts(const char *s) {
    if (!s)
        return EOF;
    sys_write(1, s, strlen(s));
    putchar('\n');
    return 0;
}

int vsnprintf(char *str, size_t size, const char *format, va_list ap) {
    if (!format || size == 0)
        return 0;

    size_t idx = 0;
    while (*format && idx + 1 < size) {
        if (*format != '%') {
            str[idx++] = *format++;
            continue;
        }

        format++;
        if (*format == '%') {
            str[idx++] = '%';
            format++;
            continue;
        }

        if (*format == 's') {
            const char *s = va_arg(ap, const char *);
            if (!s)
                s = "(null)";
            while (*s && idx + 1 < size) {
                str[idx++] = *s++;
            }
            format++;
        } else if (*format == 'd' || *format == 'i') {
            int val = va_arg(ap, int);
            char num_buf[32];
            int nidx = 0;
            if (val == 0) {
                num_buf[nidx++] = '0';
            } else {
                int neg = 0;
                if (val < 0) {
                    neg = 1;
                    val = -val;
                }
                while (val > 0 && nidx < 30) {
                    num_buf[nidx++] = '0' + (val % 10);
                    val /= 10;
                }
                if (neg)
                    num_buf[nidx++] = '-';
            }
            while (nidx > 0 && idx + 1 < size) {
                str[idx++] = num_buf[--nidx];
            }
            format++;
        } else if (*format == 'x' || *format == 'X' || *format == 'p') {
            unsigned long long val =
                (*format == 'p') ? va_arg(ap, unsigned long long) : va_arg(ap, unsigned int);
            char num_buf[32];
            int nidx = 0;
            if (val == 0) {
                num_buf[nidx++] = '0';
            } else {
                while (val > 0 && nidx < 30) {
                    int d = val & 0xF;
                    num_buf[nidx++] = (d < 10) ? ('0' + d) : ('a' + d - 10);
                    val >>= 4;
                }
            }
            while (nidx > 0 && idx + 1 < size) {
                str[idx++] = num_buf[--nidx];
            }
            format++;
        } else if (*format == 'c') {
            char c = (char)va_arg(ap, int);
            str[idx++] = c;
            format++;
        } else {
            str[idx++] = *format++;
        }
    }
    str[idx] = '\0';
    return (int)idx;
}

int snprintf(char *str, size_t size, const char *format, ...) {
    va_list ap;
    va_start(ap, format);
    int ret = vsnprintf(str, size, format, ap);
    va_end(ap);
    return ret;
}

int sprintf(char *str, const char *format, ...) {
    va_list ap;
    va_start(ap, format);
    int ret = vsnprintf(str, 65536, format, ap);
    va_end(ap);
    return ret;
}

int vprintf(const char *format, va_list ap) {
    char buf[1024];
    int len = vsnprintf(buf, sizeof(buf), format, ap);
    if (len > 0)
        sys_write(1, buf, (size_t)len);
    return len;
}

int printf(const char *format, ...) {
    va_list ap;
    va_start(ap, format);
    int ret = vprintf(format, ap);
    va_end(ap);
    return ret;
}
