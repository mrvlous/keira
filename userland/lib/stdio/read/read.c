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

size_t fread(void *ptr, size_t size, size_t nmemb, FILE *stream) {
    if (!ptr || size == 0 || nmemb == 0 || !stream)
        return 0;

    size_t total_bytes = size * nmemb;
    size_t bytes_read = 0;
    char *dst = (char *)ptr;

    if (stream->wpos > 0)
        fflush(stream);

    if (stream->buf_mode == _IONBF || !stream->buf || stream->buf_size == 0) {
        while (bytes_read < total_bytes) {
            ssize_t n = sys_read(stream->fd, dst + bytes_read, total_bytes - bytes_read);
            if (n <= 0) {
                if (n == 0)
                    stream->eof = 1;
                else
                    stream->error = 1;
                break;
            }
            bytes_read += (size_t)n;
        }
        return bytes_read / size;
    }

    while (bytes_read < total_bytes) {
        if (stream->rpos < stream->rend) {
            size_t avail = stream->rend - stream->rpos;
            size_t needed = total_bytes - bytes_read;
            size_t take = (avail < needed) ? avail : needed;
            memcpy(dst + bytes_read, stream->buf + stream->rpos, take);
            stream->rpos += take;
            bytes_read += take;
            continue;
        }

        stream->rpos = 0;
        stream->rend = 0;

        if ((total_bytes - bytes_read) >= stream->buf_size) {
            ssize_t n = sys_read(stream->fd, dst + bytes_read, total_bytes - bytes_read);
            if (n <= 0) {
                if (n == 0)
                    stream->eof = 1;
                else
                    stream->error = 1;
                break;
            }
            bytes_read += (size_t)n;
            break;
        }

        ssize_t n = sys_read(stream->fd, stream->buf, stream->buf_size);
        if (n <= 0) {
            if (n == 0)
                stream->eof = 1;
            else
                stream->error = 1;
            break;
        }
        stream->rend = (size_t)n;
    }

    return bytes_read / size;
}

int fgetc(FILE *stream) {
    unsigned char ch;
    if (fread(&ch, 1, 1, stream) == 1)
        return (int)ch;
    return EOF;
}

char *fgets(char *s, int size, FILE *stream) {
    if (!s || size <= 1 || !stream)
        return NULL;

    int idx = 0;
    while (idx < size - 1) {
        int c = fgetc(stream);
        if (c == EOF) {
            if (idx == 0)
                return NULL;
            break;
        }
        s[idx++] = (char)c;
        if (c == '\n')
            break;
    }
    s[idx] = '\0';
    return s;
}
