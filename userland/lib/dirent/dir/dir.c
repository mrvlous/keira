/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <stdlib.h>
#include <string.h>
#include <syscall.h>
#include <unistd.h>

DIR *opendir(const char *name) {
    if (!name) {
        errno = EFAULT;
        return NULL;
    }

    int fd = open(name, O_RDONLY, 0);
    if (fd < 0) {
        return NULL;
    }

    DIR *dirp = (DIR *)malloc(sizeof(DIR));
    if (!dirp) {
        close(fd);
        errno = ENOMEM;
        return NULL;
    }

    dirp->fd = fd;
    dirp->index = 0;
    memset(&dirp->current, 0, sizeof(struct dirent));
    return dirp;
}

struct dirent *readdir(DIR *dirp) {
    if (!dirp || dirp->fd < 0) {
        errno = EBADF;
        return NULL;
    }

    int ret = sys_getdents(dirp->fd, &dirp->current, sizeof(struct dirent));
    if (ret > 0) {
        dirp->index++;
        return &dirp->current;
    }

    return NULL;
}

int closedir(DIR *dirp) {
    if (!dirp || dirp->fd < 0) {
        errno = EBADF;
        return -1;
    }

    int ret = close(dirp->fd);
    free(dirp);
    return ret;
}
