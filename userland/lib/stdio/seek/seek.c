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
#include <syscall.h>
#include <unistd.h>

int fseek(FILE *stream, long offset, int whence) {
    if (!stream)
        return -1;

    fflush(stream);
    stream->rpos = 0;
    stream->rend = 0;
    stream->eof = 0;

    off_t res = sys_lseek(stream->fd, (off_t)offset, whence);
    if (res < 0) {
        stream->error = 1;
        return -1;
    }
    return 0;
}

long ftell(FILE *stream) {
    if (!stream)
        return -1;

    off_t pos = sys_lseek(stream->fd, 0, SEEK_CUR);
    if (pos < 0)
        return -1;

    if (stream->rend > stream->rpos)
        pos -= (off_t)(stream->rend - stream->rpos);
    if (stream->wpos > 0)
        pos += (off_t)stream->wpos;

    return (long)pos;
}

int feof(FILE *stream) {
    return stream ? stream->eof : 0;
}

int ferror(FILE *stream) {
    return stream ? stream->error : 0;
}

void clearerr(FILE *stream) {
    if (stream) {
        stream->eof = 0;
        stream->error = 0;
    }
}
