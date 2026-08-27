<!-- SPDX-License-Identifier: GPL-2.0-only -->

# System & Diagnostic Shell Commands

This document details all native commands in Keira Kernel related to hardware diagnostics, memory metrics, CPU features, system power state, and service lifecycle.

---

## Command Reference Table

| Command | Syntax | Description |
| :--- | :--- | :--- |
| `system` | `system [info \| version \| uname]` | Display kernel version, architecture, build timestamp, and compiler version |
| `runtime` | `runtime [status \| stats]` | Display active kernel uptime, tick rate, and context switch frequency |
| `memory` | `memory [info \| pmm \| heap \| map]` | Inspect physical frame allocations, kernel heap stats, and virtual memory layout |
| `cpu` | `cpu [info \| topology \| features]` | Display CPU vendor, brand, core count, APIC IDs, and SSE/AVX capability flags |
| `time` | `time [get \| set <epoch> \| rtc]` | Query Real-Time Clock (RTC) and hardware PIT uptime |
| `env` | `env [list \| set <key> <val> \| get <key>]` | Inspect and manipulate environment variables (`$PATH`, `$USER`, `$HOME`) |
| `hostname` | `hostname [get \| set <name>]` | Query or update system hostname |
| `power` | `power [shutdown \| reboot \| sleep]` | ACPI hardware power control and soft reboot |
| `reset` | `reset` | Trigger immediate hardware CPU reset via keyboard controller port `0x64` |
| `sync` | `sync` | Flush all dirty filesystem cache buffers to physical block media |
| `service` | `service [list \| start <svc> \| stop <svc>]` | Inspect and control background system service daemons |
| `syslog` | `syslog [tail \| clear \| dump]` | Inspect in-memory kernel ring buffer logs (`klog`) |
| `unwind` | `unwind` | Display stack trace frame unwinding for the current execution context |

---

## Detailed Usage

### `memory info`
Queries the Physical Memory Manager (PMM) and heap allocator:
```bash
keira> memory info
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

### `cpu info`
Queries CPUID instruction features:
```bash
keira> cpu info
CPU 0:
  Vendor        : GenuineIntel (Intel QEMU Virtual CPU)
  Architecture  : x86_64 Long Mode
  Features      : FPU, VME, DE, PSE, TSC, MSR, PAE, MCE, CX8, APIC, SEP, MTRR, PGE, MCA, CMOV, PAT, PSE36, SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, NX, LM
```
