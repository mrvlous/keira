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
 * Keira User-Space Standard I/O Implementation
 *
 * Provides formatted output operations (printf) for user space applications
 * by wrapping the sys_print_char system call.
 */

#include "stdio.h"

#include "syscall.h"

#include <stdarg.h>

/**
 * print_str - Write null-terminated string to stdout.
 * @str: Pointer to null-terminated string.
 *
 * Return: Number of characters written.
 */
static int print_str(const char *str) {
    int count = 0;
    while (*str) {
        sys_print_char(*str++);
        count++;
    }
    return count;
}

/**
 * print_int - Format signed 64-bit integer into ASCII decimal representation.
 * @val: Signed integer value.
 *
 * Return: Number of characters printed.
 */
static int print_int(long val) {
    char buf[24];
    int i = 0;
    int count = 0;
    int is_neg = 0;
    if (val < 0) {
        is_neg = 1;
        val = -val;
    } else if (val == 0) {
        sys_print_char('0');
        return 1;
    }
    while (val > 0) {
        buf[i++] = '0' + (val % 10);
        val /= 10;
    }
    if (is_neg) {
        sys_print_char('-');
        count++;
    }
    while (i > 0) {
        sys_print_char(buf[--i]);
        count++;
    }
    return count;
}

/**
 * print_hex - Format unsigned integer into ASCII hexadecimal format (0x...).
 * @val: Unsigned integer value.
 *
 * Return: Number of characters printed.
 */
static int print_hex(unsigned long val) {
    char hex_chars[] = "0123456789ABCDEF";
    char buf[16];
    int i = 0;
    int count = 0;
    if (val == 0) {
        return print_str("0x0");
    }
    while (val > 0) {
        buf[i++] = hex_chars[val & 0xF];
        val >>= 4;
    }
    count += print_str("0x");
    while (i > 0) {
        sys_print_char(buf[--i]);
        count++;
    }
    return count;
}

/**
 * printf - Format and print string to user terminal output.
 * @fmt: Format specification string (%s, %d, %x, %c, %%).
 *
 * Return: Total number of characters written to stdout.
 */
int printf(const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);
    int count = 0;
    while (*fmt) {
        if (*fmt == '%') {
            fmt++;
            if (*fmt == '\0') {
                break;
            }
            switch (*fmt) {
            case 's': {
                const char *s = va_arg(args, const char *);
                if (s) {
                    count += print_str(s);
                } else {
                    count += print_str("(null)");
                }
                break;
            }
            case 'd': {
                int d = va_arg(args, int);
                count += print_int(d);
                break;
            }
            case 'x': {
                unsigned int x = va_arg(args, unsigned int);
                count += print_hex(x);
                break;
            }
            case 'c': {
                char c = (char)va_arg(args, int);
                sys_print_char(c);
                count++;
                break;
            }
            case '%': {
                sys_print_char('%');
                count++;
                break;
            }
            default:
                sys_print_char('%');
                sys_print_char(*fmt);
                count += 2;
                break;
            }
        } else {
            sys_print_char(*fmt);
            count++;
        }
        fmt++;
    }
    va_end(args);
    return count;
}
