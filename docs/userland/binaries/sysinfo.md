<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `sysinfo.elf` System Telemetry Utility

Displays hardware, operating system, and runtime statistics retrieved via system calls and ProcFS.

---

## Telemetry Fields

* **OS Release**: Kernel name, version, architecture (`i686` or `x86_64`), build date.
* **CPU Information**: Model string, core count, frequency, supported extensions (SSE, AVX).
* **Memory Usage**: Total physical RAM, available memory, kernel heap usage, swap status.
* **Storage Devices**: Detected ATA/AHCI drives, mounted VFS partitions, filesystem free space.
* **Uptime**: Monotonic system uptime and clock synchronization status.
