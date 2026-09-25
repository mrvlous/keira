<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel Main Loop & Executive Runtime

The kernel main loop coordinates idle execution, hardware interrupts, and background service maintenance.

---

## Runtime Cycle

1. **Subsystem Initializer**: Brings up filesystems, networking, and task scheduler.
2. **Userland Spawn**: Launches the initial interactive shell process (`/system/bin/shell`).
3. **Executive Loop**: Executes the kernel idle thread (`HLT` loop), servicing interrupts and background daemons.
