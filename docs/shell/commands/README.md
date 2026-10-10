<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Keira Built-in Command Reference Manual

Keira Shell provides 15 native built-in emergency diagnostic commands partitioned into 5 functional modules. Standard user utilities (`ps`, `kill`, `hostname`, `clear`, `dmesg`, `df`, `cat`, `ls`, `fetch`, `kcc`, `sh` and `init`) are hosted in Ring 3 userspace (`/bin`).

---

## Category Directory Map

| Category | Path | Scope | Command Count |
| :--- | :--- | :--- | :--- |
| **System Diagnostics & Power** | [`sys.md`](sys.md) | CPU telemetry, APIC, memory, reboot, poweroff, sync, syslog, system, unwind and watchpoint | 10 commands |
| **Device & Driver Registry** | [`dev.md`](dev.md) | PCI bus enumeration and active kernel driver registry | 2 commands |
| **Storage & Disk Geometry** | [`fs.md`](fs.md) | Raw block storage devices and partition geometry | 1 command |
| **Process & Direct Launch** | [`proc.md`](proc.md) | Direct ELF binary execution from storage | 1 command |
| **Emergency Utilities** | [`util.md`](util.md) | Interactive emergency help index | 1 command |
