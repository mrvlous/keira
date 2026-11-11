/* SPDX-License-Identifier: GPL-2.0-only */

#ifndef _SYSINFO_H
#define _SYSINFO_H

#include <sys/types.h>

/**
 * Queries and prints process credentials, uptime and kernel hostname
 * telemetry.
 *
 * @return 0 on success, or non-zero error code.
 */
int display_system_info(void);

#endif /* _SYSINFO_H */
