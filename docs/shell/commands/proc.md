<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Process & Task Control Shell Commands

This document details all native commands in Keira Kernel related to task monitoring, scheduling, signal dispatching, job control, cgroups, and futex synchronizations.

---

## Command Reference Table

| Command | Syntax | Description |
| :--- | :--- | :--- |
| `tasks` | `tasks` | Display running kernel and userland tasks, PIDs, states, and CPU runtime |
| `kill` | `kill <pid> [sig]` | Send POSIX signal (`SIGKILL`, `SIGTERM`, `SIGINT`) to a target task |
| `stop` | `stop <pid>` | Pause task execution (`SIGSTOP`) |
| `bg` | `bg <job_id>` | Resume stopped job in the background |
| `fg` | `fg <job_id>` | Bring background job to the foreground |
| `jobs` | `jobs` | List active background and stopped shell job entries |
| `cgroups` | `cgroups [list \| create <cg> \| set]` | Inspect and configure cgroups resource limits (CPU shares, memory max) |
| `futex` | `futex [status \| wait \| wake]` | Inspect active userland fast userspace mutex wait queues |
| `eventfd` | `eventfd [list \| create \| read]` | Manage kernel `eventfd` notification descriptors |
| `perf` | `perf [stat \| top \| counters]` | Sample CPU performance monitoring unit (PMU) hardware counters |
| `timer` | `timer [list \| set <ms> \| cancel]` | Manage POSIX high-resolution timer queues |
| `run` | `run <elf_path> [args...]` | Execute an ELF binary in isolated Ring 3 userland |

---

## Detailed Usage

### `tasks`
Displays the real-time preemptive scheduler runqueue:
```bash
keira> tasks
  PID  PPID  STATE     PRIO  NAME          MEM (KB)  CPU TIME
  0    0     RUNNING   0     [idle]        0         1240 ms
  1    0     SLEEPING  10    [init]        64        15 ms
  2    1     RUNNING   20    [shell]       128       450 ms
  3    2     RUNNING   15    kcc.elf       512       80 ms
```

### `run <elf_path>`
Loads and executes a dynamic ELF binary within an isolated virtual address space:
```bash
keira> run /system/bin/kcc.elf -v
Keira C Compiler (KCC) v0.36.0
Target: x86_64-keira-none
Ready.
```
