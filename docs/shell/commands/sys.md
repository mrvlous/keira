<!-- SPDX-License-Identifier: GPL-2.0-only -->

# System Information & Control Commands

The `sys` command suite provides inspection, telemetry, power management, and hardware diagnostics for the Keira Kernel.

---

## Command Reference

| Command | Syntax | Description | Flags / Options |
| :--- | :--- | :--- | :--- |
| `cpu` | `cpu` | Display processor model, feature flags, APIC clock, context switches, and ticks | `-h, --help` |
| `hostname` | `hostname [name]` | Query current node hostname or configure a new identity | `-h, --help` |
| `initrd` | `initrd` | Inspect Multiboot2 initramfs RAM disk location, byte size, and header | `-h, --help` |
| `memory` | `memory [-t\|-p]` | Display physical memory (PMM) frames and kernel dynamic heap usage | `-t, --test`: Heap & slab stress test<br>`-p, --paging`: Demand paging & #PF telemetry<br>`-h, --help`: Usage info |
| `perf` | `perf` | Query CPU cycle performance counters and scheduling latency metrics | `-h, --help` |
| `power` | `power [status\|shutdown]` | Query ACPI S0-S5 states, motherboard MADT topology, or soft-off power down | `-h, --help` |
| `reset` | `reset` | Reboot kernel and motherboard via PS/2 controller pulse | `-h, --help` |
| `runtime` | `runtime` | Display kernel uptime and total timer interrupts processed | `-h, --help` |
| `smp` | `smp` | Inspect multi-core APIC topology and secondary processor status | `-h, --help` |
| `swap` | `swap` | Query disk-backed virtual memory paging partition status | `-h, --help` |
| `syslog` | `syslog` | Read kernel circular ring buffer diagnostic log (`dmesg`) | `-h, --help` |
| `system` | `system` | Print kernel version, architecture, compiler target, and build info | `-h, --help` |
| `time` | `time` | Read CMOS Real Time Clock (RTC) date, time, and UTC timestamp | `-h, --help` |
| `timer` | `timer` | Display Programmable Interval Timer (PIT) frequency and timer descriptors | `-h, --help` |
| `unwind` | `unwind` | Display stack frame trace and symbol unwind diagnostic test | `-h, --help` |
| `watchpoint` | `watchpoint` | Inspect hardware debug registers (DR0-DR3) and watchpoint traps | `-h, --help` |
