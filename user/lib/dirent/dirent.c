/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#include <dirent.h>
#include <errno.h>
#include <malloc.h>
#include <string.h>

static DIR static_dir;

DIR *opendir(const char *name) {
    if (!name) {
        errno = EFAULT;
        return NULL;
    }
    static_dir.fd = 3;
    static_dir.index = 0;
    memset(&static_dir.current, 0, sizeof(struct dirent));
    return &static_dir;
}

struct dirent *readdir(DIR *dirp) {
    if (!dirp) {
        errno = EBADF;
        return NULL;
    }
    if (dirp->index == 0) {
        dirp->current.d_ino = 1;
        dirp->current.d_type = DT_DIR;
        strcpy(dirp->current.d_name, ".");
        dirp->index++;
        return &dirp->current;
    } else if (dirp->index == 1) {
        dirp->current.d_ino = 2;
        dirp->current.d_type = DT_DIR;
        strcpy(dirp->current.d_name, "..");
        dirp->index++;
        return &dirp->current;
    }
    return NULL;
}

int closedir(DIR *dirp) {
    (void)dirp;
    return 0;
}
