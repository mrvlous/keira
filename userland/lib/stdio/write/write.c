/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <stdio.h>
#include <string.h>
#include <syscall.h>

size_t fwrite(const void *ptr, size_t size, size_t nmemb, FILE *stream) {
    if (!ptr || size == 0 || nmemb == 0 || !stream)
        return 0;

    size_t total_bytes = size * nmemb;
    size_t bytes_written = 0;
    const char *src = (const char *)ptr;

    stream->rpos = 0;
    stream->rend = 0;

    if (stream->buf_mode == _IONBF || !stream->buf || stream->buf_size == 0) {
        while (bytes_written < total_bytes) {
            ssize_t n = sys_write(stream->fd, src + bytes_written, total_bytes - bytes_written);
            if (n <= 0) {
                stream->error = 1;
                break;
            }
            bytes_written += (size_t)n;
        }
        return bytes_written / size;
    }

    while (bytes_written < total_bytes) {
        size_t space = stream->buf_size - stream->wpos;
        if (space == 0) {
            if (fflush(stream) != 0)
                break;
            space = stream->buf_size;
        }

        size_t needed = total_bytes - bytes_written;
        size_t put = (space < needed) ? space : needed;
        memcpy(stream->buf + stream->wpos, src + bytes_written, put);
        stream->wpos += put;
        bytes_written += put;

        if (stream->buf_mode == _IOLBF) {
            for (size_t i = 0; i < put; i++) {
                if (src[bytes_written - put + i] == '\n') {
                    fflush(stream);
                    break;
                }
            }
        }
    }

    return bytes_written / size;
}

int fputc(int c, FILE *stream) {
    unsigned char ch = (unsigned char)c;
    if (fwrite(&ch, 1, 1, stream) == 1)
        return (int)ch;
    return EOF;
}

int fputs(const char *s, FILE *stream) {
    if (!s || !stream)
        return EOF;
    size_t len = strlen(s);
    if (len == 0)
        return 0;
    size_t written = fwrite(s, 1, len, stream);
    return (written == len) ? 0 : EOF;
}

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
