.. SPDX-License-Identifier: GPL-2.0-only


Dynamic File Dispatcher
=======================

Generates live runtime contents upon ``read`` operations for virtual nodes (``uptime``, ``meminfo``, ``buddyinfo``, ``waitqueue``, ``workqueues``, ``daemons``, ``cpuinfo``, ``version``, ``loadavg``, ``cmdline``, ``slabinfo``, ``vmstat``, ``self``). System command-line reflection (``/proc/cmdline``) exposes the active Multiboot2 kernel boot arguments captured from ``TAG_TYPE_CMDLINE``.

Virtual directory enumeration (``SYS_GETDENTS``) is supported over ``/proc``, synthesizing canonical directory entries for all telemetry nodes, active dynamic PID subdirectories (``/proc/<pid>/``) and per-process telemetry nodes (``stat``, ``statm``, ``status``, ``cmdline``, ``maps``, ``limits``, ``mounts``).
