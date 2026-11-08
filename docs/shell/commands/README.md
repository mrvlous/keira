<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Keira Built-in Command Reference Manual

Keira Shell provides 32 native built-in emergency diagnostic commands partitioned into 5 functional modules. Standard user utilities (`ps`, `kill`, `hostname`, `dmesg`, `df`, `cat`, `ls`, `fetch`, `kcc`, `sh` and `init`) are hosted in Ring 3 userspace (`/bin`). Terminal console display clearing in Ring 0 is handled via `wipe`.

---

## Category Directory Map

| Category | Path | Scope | Command Count |
| :--- | :--- | :--- | :--- |
| **System Diagnostics & Power** | [`sys.md`](sys.md) | CPU telemetry, APIC, IRQ routing, control registers, memory, pipes, polling, reboot, poweroff, sync, syslog, system, unwind, watchpoint and wipe | 22 commands |
| **Device & Driver Registry** | [`dev.md`](dev.md) | PCI bus enumeration and active kernel driver registry | 2 commands |
| **Storage & Disk Geometry** | [`fs.md`](fs.md) | Raw block storage devices, partition geometry, dcache and icache | 4 commands |
| **Process & Direct Launch** | [`proc.md`](proc.md) | Direct ELF binary execution, process inspection and interactive shell launch | 3 commands |
| **Emergency Utilities** | [`util.md`](util.md) | Interactive emergency help index | 1 command |
