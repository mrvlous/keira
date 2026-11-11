// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

#ifndef _UAPI_SYS_MOUNT_H
#define _UAPI_SYS_MOUNT_H

#define MS_RDONLY 1
#define MS_NOSUID 2
#define MS_NODEV 4
#define MS_NOEXEC 8
#define MS_SYNCHRONOUS 16
#define MS_REMOUNT 32
#define MS_MANDLOCK 64
#define MS_DIRSYNC 128
#define MS_NOATIME 1024
#define MS_NODIRATIME 2048
#define MS_BIND 4096

int mount(const char *source, const char *target, const char *filesystemtype,
          unsigned long mountflags, const void *data);
int umount(const char *target);
int umount2(const char *target, int flags);

#endif /* _UAPI_SYS_MOUNT_H */
