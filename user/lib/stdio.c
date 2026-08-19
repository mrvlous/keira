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
    unsigned long uval;
    if (val < 0) {
        sys_print_char('-');
        count++;
        uval = (unsigned long)(-val);
    } else if (val == 0) {
        sys_print_char('0');
        return 1;
    } else {
        uval = (unsigned long)val;
    }
    while (uval > 0) {
        buf[i++] = (char)('0' + (uval % 10));
        uval /= 10;
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

#include "malloc.h"

FILE *fopen(const char *filename, const char *mode) {
    if (!filename || !mode)
        return NULL;
    int write_mode = 0;
    if (mode[0] == 'w' || mode[0] == 'a') {
        write_mode = (mode[0] == 'a') ? 2 : 1;
    }
    int fd = sys_open(filename, write_mode);
    if (fd < 0)
        return NULL;

    FILE *f = (FILE *)malloc(sizeof(FILE));
    if (!f) {
        sys_close(fd);
        return NULL;
    }
    f->fd = fd;
    f->flags = write_mode;
    f->pos = 0;
    f->eof = 0;
    f->err = 0;
    return f;
}

size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream) {
    if (!ptr || size == 0 || nmemb == 0 || !stream)
        return 0;
    int bytes_to_read = size * nmemb;
    int res = sys_read(stream->fd, ptr, bytes_to_read);
    if (res <= 0) {
        stream->eof = 1;
        return 0;
    }
    stream->pos += res;
    return res / size;
}

size_t fwrite(const void *ptr, size_t size, size_t nmemb, FILE *stream) {
    if (!ptr || size == 0 || nmemb == 0 || !stream)
        return 0;
    int bytes_to_write = size * nmemb;
    int res = sys_write(stream->fd, ptr, bytes_to_write);
    if (res < 0) {
        stream->err = 1;
        return 0;
    }
    stream->pos += res;
    return res / size;
}

int fclose(FILE *stream) {
    if (!stream)
        return -1;
    int res = sys_close(stream->fd);
    free(stream);
    return res;
}

int fseek(FILE *stream, long offset, int whence) {
    if (!stream)
        return -1;
    (void)whence;
    int res = sys_seek(stream->fd, (unsigned long)offset);
    if (res >= 0) {
        stream->pos = offset;
        stream->eof = 0;
        return 0;
    }
    return -1;
}

long ftell(FILE *stream) {
    if (!stream)
        return -1;
    return stream->pos;
}

int feof(FILE *stream) {
    if (!stream)
        return 1;
    return stream->eof;
}

int remove(const char *filename) {
    (void)filename;
    return 0;
}