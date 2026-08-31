/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Operating System Kernel
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _SYS_STAT_H
#define _SYS_STAT_H

#include <sys/types.h>

#define S_IFMT 0170000
#define S_IFSOCK 0140000
#define S_IFLNK 0120000
#define S_IFREG 0100000
#define S_IFBLK 0060000
#define S_IFDIR 0040000
#define S_IFCHR 0020000
#define S_IFIFO 0010000

#define S_ISREG(m) (((m) & S_IFMT) == S_IFREG)
#define S_ISDIR(m) (((m) & S_IFMT) == S_IFDIR)
#define S_ISCHR(m) (((m) & S_IFMT) == S_IFCHR)
#define S_ISBLK(m) (((m) & S_IFMT) == S_IFBLK)

#define S_IRWXU 00700
#define S_IRUSR 00400
#define S_IWUSR 00200
#define S_IXUSR 00100

#define S_IRWXG 00070
#define S_IRGRP 00040
#define S_IWGRP 00020
#define S_IXGRP 00010

#define S_IRWXO 00007
#define S_IROTH 00004
#define S_IWOTH 00002
#define S_IXOTH 00001

struct stat {
    uint64_t st_dev;
    uint64_t st_ino;
    mode_t st_mode;
    uint32_t st_nlink;
    uid_t st_uid;
    gid_t st_gid;
    uint64_t st_rdev;
    off_t st_size;
    uint64_t st_blksize;
    uint64_t st_blocks;
    time_t st_atime;
    time_t st_mtime;
    time_t st_ctime;
};

int stat(const char *pathname, struct stat *statbuf);
int fstat(int fd, struct stat *statbuf);
int mkdir(const char *pathname, mode_t mode);
int chmod(const char *pathname, mode_t mode);

#endif /* _SYS_STAT_H */
