<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `keira-task` - Multitasking & Process Scheduling

The `keira-task` crate implements preemptive round-robin task scheduling, thread context switching, resource control groups (cgroups), Mandatory Access Control (MAC), Seccomp BPF syscall filters, and POSIX signal delivery.

## Submodules

- [`types.md`](types.md): Process control blocks and thread states.
- [`scheduler.md`](scheduler.md): Preemptive round-robin scheduler.
- [`cgroups.md`](cgroups.md): Resource control groups & PID namespaces.
- [`security.md`](security.md): MAC path security & Seccomp filters.
- [`signal.md`](signal.md): POSIX signals & terminal job control table.
