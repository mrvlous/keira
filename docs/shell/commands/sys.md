<!-- SPDX-License-Identifier: GPL-2.0-only -->

# System Information & Control Commands

The `sys` command suite provides emergency telemetry, power management and hardware diagnostics for the Keira Kernel.

---

## Command Reference

| Command | Syntax | Description | Flags / Options |
| :--- | :--- | :--- | :--- |
| `aslrinfo` | `aslrinfo [-v] [0\|1\|2]` | Display and configure Address Space Layout Randomization (ASLR) and hardware entropy status (alias: `aslr`) | `-v, --verbose`: Display per-task randomized layout table<br>`-h, --help`: Usage info |
| `cpu` | `cpu` | Display processor model, feature flags, APIC clock, context switches and ticks | `-h, --help` |
| `irq` | `irq` | Display IDT interrupt vector routing and processed hit counters | `-h, --help` |
| `memory` | `memory [-t\|-p]` | Display physical memory (PMM) frames and kernel dynamic heap usage (alias: `mem`) | `-t, --test`: Heap & slab stress test<br>`-p, --paging`: Demand paging & #PF telemetry<br>`-h, --help`: Usage info |
| `power` | `power [status\|shutdown]` | Query ACPI S0-S5 states, motherboard MADT topology or soft-off power down (aliases: `poweroff`, `shutdown`) | `-h, --help` |
| `regs` | `regs` | Display CPU control registers (CR0-CR4) and execution flags | `-h, --help` |
| `reset` | `reset` | Reboot kernel and motherboard via PS/2 controller pulse (alias: `reboot`) | `-h, --help` |
| `secinfo` | `secinfo [-v]` | Display hardware exploit mitigations, memory boundary protections and #PF telemetry (alias: `mitigations`) | `-v, --verbose`: Display VMM fault counters<br>`-h, --help`: Usage info |
| `smp` | `smp` | Inspect multi-core APIC topology and secondary processor status | `-h, --help` |
| `sync` | `sync` | Flush dirty filesystem cache sectors to physical disk | `-h, --help` |
| `syslog` | `syslog [-b]` | Read system event and kernel log records (alias: `dmesg`) | `-b, --boot`: Display boot record<br>`-h, --help`: Usage info |
| `system` | `system` | Print kernel version, architecture, compiler target and build info | `-h, --help` |
| `unwind` | `unwind` | Display stack frame trace and symbol unwind diagnostic test | `-h, --help` |
| `watchpoint` | `watchpoint` | Inspect hardware debug registers (DR0-DR3) and watchpoint traps | `-h, --help` |
| `wipe` | `wipe` | Clear and reset the VGA text console display buffer | `-h, --help` |


