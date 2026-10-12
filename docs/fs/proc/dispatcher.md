<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Dynamic File Dispatcher

Generates synthetic file contents upon `read` operations for virtual nodes (`uptime`, `meminfo`, `cpuinfo`, `version`, `loadavg`, `cmdline`, `self`).

Virtual directory enumeration (`SYS_GETDENTS`) is supported over `/proc`, synthesising canonical directory entries for all telemetry nodes and subdirectories.
