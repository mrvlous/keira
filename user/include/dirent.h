/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _DIRENT_H
#define _DIRENT_H

#include <stdint.h>
#include <sys/types.h>

#define DT_UNKNOWN 0
#define DT_FIFO 1
#define DT_CHR 2
#define DT_DIR 4
#define DT_BLK 6
#define DT_REG 8
#define DT_LNK 10
#define DT_SOCK 12

struct dirent {
    uint64_t d_ino;
    off_t d_off;
    uint16_t d_reclen;
    uint8_t d_type;
    char d_name[256];
};

typedef struct {
    int fd;
    int index;
    struct dirent current;
} DIR;

DIR *opendir(const char *name);
struct dirent *readdir(DIR *dirp);
int closedir(DIR *dirp);

#endif /* _DIRENT_H */
