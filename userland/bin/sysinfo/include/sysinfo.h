/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Keira Kernel - Freestanding Kernel from Scratch
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 */

#ifndef _SYSINFO_H
#define _SYSINFO_H

#include <sys/types.h>

/**
 * Queries and prints process credentials, uptime, and kernel hostname
 * telemetry.
 *
 * @return 0 on success, or non-zero error code.
 */
int display_system_info(void);

#endif /* _SYSINFO_H */
