<!-- SPDX-License-Identifier: GPL-2.0-only -->

# System Information & Control Commands

The `sys` command suite provides emergency telemetry, power management and hardware diagnostics for the Keira Kernel.

---

## Command Reference

| Command | Syntax | Description | Flags / Options |
| :--- | :--- | :--- | :--- |
| `cpu` | `cpu` | Display processor model, feature flags, APIC clock, context switches and ticks | `-h, --help` |
| `memory` | `memory [-t\|-p]` | Display physical memory (PMM) frames and kernel dynamic heap usage | `-t, --test`: Heap & slab stress test<br>`-p, --paging`: Demand paging & #PF telemetry<br>`-h, --help`: Usage info |
| `power` | `power [status\|shutdown]` | Query ACPI S0-S5 states, motherboard MADT topology or soft-off power down (aliases: `poweroff`, `shutdown`) | `-h, --help` |
| `reset` | `reset` | Reboot kernel and motherboard via PS/2 controller pulse (alias: `reboot`) | `-h, --help` |
| `smp` | `smp` | Inspect multi-core APIC topology and secondary processor status | `-h, --help` |
| `sync` | `sync` | Flush dirty filesystem cache sectors to physical disk | `-h, --help` |
| `syslog` | `syslog [-b]` | Read system event and kernel log records | `-b, --boot`: Display boot record<br>`-h, --help`: Usage info |
| `system` | `system` | Print kernel version, architecture, compiler target and build info | `-h, --help` |
| `unwind` | `unwind` | Display stack frame trace and symbol unwind diagnostic test | `-h, --help` |
| `watchpoint` | `watchpoint` | Inspect hardware debug registers (DR0-DR3) and watchpoint traps | `-h, --help` |
