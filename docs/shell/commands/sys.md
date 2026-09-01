<!-- SPDX-License-Identifier: GPL-2.0-only -->

# System & Diagnostic Shell Commands

This document details all native commands in Keira Kernel related to hardware diagnostics, memory metrics, CPU features, system power state, and service lifecycle.

---

## Command Reference Table

| Command | Syntax | Status | Description |
| :--- | :--- | :--- | :--- |
| `system` | `system [-v] [-u] [-s]` | `[Active]` | Display kernel specifications, architecture, memory stats, and uptime |
| `cpu` | `cpu` | `[Active]` | Display CPU vendor signature string (e.g. AuthenticAMD, GenuineIntel) |
| `runtime` | `runtime` | `[Active]` | Display time elapsed since system boot in milliseconds |
| `time` | `time` | `[Active]` | Query Real-Time Clock (RTC) date and time in UTC |
| `memory` | `memory` | `[Active]` | Inspect physical frame allocations (PMM) and heap memory telemetry |
| `service` | `service [list \| start <svc> \| stop <svc> \| edit <svc>]` | `[Active]` | Inspect and control `ksvc` background service daemons (httpd, syslogd, syncd, watchdogd) |
| `env` | `env [list \| set <k> <v> \| get <k>]` | `[Active]` | Inspect and manipulate shell runtime environment variables |
| `hostname` | `hostname [get \| set <name>]` | `[Active]` | Query or update persistent system hostname in `/config/sys/hostname` |
| `power` | `power [status \| acpi \| shutdown \| reboot]` | `[Active]` | Query ACPI power state, initiate S5 soft-off shutdown, or reboot CPU |
| `poweroff` | `poweroff` | `[Active]` | Direct alias for `power shutdown` to power off hardware via ACPI S5 |
| `reset` | `reset` | `[Active]` | Trigger immediate bare-metal CPU reboot via PS/2 controller port `0x64` |
| `reboot` | `reboot` | `[Active]` | Direct alias for `reset` to reboot machine via PS/2 fast reset |
| `sync` | `sync` | `[Active]` | Flush dirty filesystem block cache sectors to physical storage media |
| `syslog` | `syslog [dmesg]` | `[Active]` | Read circular kernel syslog dmesg diagnostic log buffer (Syscall 44) |
| `unwind` | `unwind` | `[Active]` | Walk active kernel callstack frame pointers (RBP/RIP) for backtrace (Syscall 37) |

---

## Detailed Usage

### `memory`
Queries the Physical Memory Manager (PMM) and kernel heap allocator:
```bash
keira> memory
Physical Memory Manager (PMM):
  Total Memory  : 128 MB (32768 frames)
  Allocated     : 24.5 MB (6272 frames)
  Free Memory   : 103.5 MB (26496 frames)

Kernel Bump/Slab Heap:
  Heap Base     : 0xFFFF800000000000
  Total Size    : 16 MB
  Used Memory   : 1.2 MB
  Allocations   : 1420 active
```

### `service list`
Displays the status of background services managed by `ksvc`:
```bash
keira> service list
Service Name  State     Port/PID  Description
[httpd]       RUNNING   Port 80   Native Background HTTP Web Server
[syslogd]     RUNNING   PID 3     System Event Logging Daemon
[syncd]       RUNNING   PID 4     Filesystem Cache Auto-Sync Daemon
[watchdogd]   RUNNING   PID 5     Kernel Health & Crash Watchdog
```
