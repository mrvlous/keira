/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _SYS_UTSNAME_H
#define _SYS_UTSNAME_H

#define _SYS_NAMELEN 65

struct utsname {
    char sysname[_SYS_NAMELEN];
    char nodename[_SYS_NAMELEN];
    char release[_SYS_NAMELEN];
    char version[_SYS_NAMELEN];
    char machine[_SYS_NAMELEN];
    char domainname[_SYS_NAMELEN];
};

int uname(struct utsname *buf);

#endif /* _SYS_UTSNAME_H */
