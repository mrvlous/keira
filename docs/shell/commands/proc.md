<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Process & Task Control Commands

The `proc` command suite provides task lifecycle control, CPU accounting and kernel IPC primitives.

> [!NOTE]
> **Pure Kernel Demarcation**: High-level C compilation is handled strictly in Ring 3 userspace via the freestanding binary [`/bin/kcc.elf`](../../userland/binaries/kcc.md). The Ring 0 supervisor console provides process inspection (`tasks`), signal delivery (`kill`), CPU cgroups and IPC telemetry.

---

## Command Reference

| Command | Syntax | Description | Flags / Options |
| :--- | :--- | :--- | :--- |
| `tasks` | `tasks [-s]` | List process table with PID, state, CPU ticks and switch counters (alias: `ps`) | `-s, --summary`: Scheduler telemetries<br>`-h, --help`: Usage info |
| `run` | `run <binary>` | Execute an ELF binary from storage or launch background worker | `-h, --help` |
| `stop` | `stop <pid>` | Suspend task execution state | `-h, --help` |
| `kill` | `kill [-sig] <pid>` | Send POSIX signals (SIGTERM, SIGKILL, SIGINT) to process | `-h, --help` |
| `cgroups` | `cgroups` | Inspect control group hierarchy, CPU quota and memory limits | `-h, --help` |
| `futex` | `futex` | Inspect active userland fast mutex wait queues and hash buckets | `-h, --help` |
| `eventfd` | `eventfd` | Query event notification file descriptors and counters | `-h, --help` |
| `epoll` | `epoll` | Display I/O event multiplexer descriptors and registered events | `-h, --help` |
| `mqueue` | `mqueue` | Query POSIX message queue descriptors, capacity and depth | `-h, --help` |
| `timer` | `timer` | Display active timer file descriptors and alarm triggers | `-h, --help` |
